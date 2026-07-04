//! autoconf — 契約・外部API不要のローカル自己学習型オートコンフィグ
//!
//! 「独自学習成長型AI」と呼べる実体を誠実に定義すると: 外部クラウドAIへの
//! 契約や通信は一切行わず、**自分自身が受けたリクエストの統計だけ**を
//! ローカルファイル(`tuning.json`)に蓄積し、その統計に基づいて
//!   - systemd unit ファイル
//!   - routes.toml (proxy_mode 用ルーティング)
//! を自動生成・自動再調整する、軽量な自己学習ループです。
//! (ディープラーニング等の大規模モデルではなく、EMA+閾値ルールによる
//!  適応制御 = "self-tuning controller" です。誇張せず明記します。)
//!
//! これにより、Apache/Nginxのように .conf を手で書く代わりに、
//! 実運用トラフィックを数分観測するだけで妥当な初期値に収束します。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningState {
    pub ema_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub sample_count: u64,
    pub worker_threads: usize,
    pub keepalive_secs: u64,
    pub last_updated_unix: u64,
}

impl Default for TuningState {
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        Self {
            ema_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            sample_count: 0,
            worker_threads: cores, // 初期値: 論理コア数 (Apache prefork の MaxRequestWorkers相当の考え方)
            keepalive_secs: 15,
            last_updated_unix: 0,
        }
    }
}

pub struct AutoConf {
    path: PathBuf,
    state: Mutex<TuningState>,
    recent_max_ms: AtomicU64, // 簡易p99用の直近最大値 (軽量実装)
}

impl AutoConf {
    /// 既存の tuning.json があれば読み込み (= 前回までの学習結果を引き継ぐ)。
    /// さらに、同一ディレクトリ配下に "aruaru-brain.json"(rclone等の同期対象)が
    /// あれば tuning.json より優先して復元する — こちらが「引っ越し後の正」となる。
    pub fn load_or_init(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let state = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let ac = Self { path, state: Mutex::new(state), recent_max_ms: AtomicU64::new(0) };
        let brain_path = ac.brain_path();
        if brain_path.exists() {
            match ac.import_brain(&brain_path) {
                Ok(true) => tracing::info!(?brain_path, "restored learning state from synced brain file (e.g. via rclone/Google Drive)"),
                Ok(false) => tracing::warn!(?brain_path, "brain file present but rejected (see warnings above) — using tuning.json instead"),
                Err(e) => tracing::warn!(?brain_path, error = %e, "failed to read brain file — using tuning.json instead"),
            }
        }
        ac
    }

    /// ポータブルスナップショットの既定パス。
    /// rclone/Insync等の同期フォルダをここに向けるだけで「賢さ」がクラウド経由で引き継がれる。
    pub fn brain_path(&self) -> PathBuf {
        self.path.parent().unwrap_or(Path::new(".")).join("aruaru-brain.json")
    }

    /// リクエスト完了毎に呼ぶ。EMAで学習し、100サンプル毎に閾値ルールで
    /// worker_threads / keepalive_secs を再調整して設定ファイルを書き出す。
    pub fn record(&self, latency_ms: f64) {
        self.recent_max_ms.fetch_max(latency_ms as u64, Ordering::Relaxed);

        let mut s = self.state.lock().unwrap();
        s.ema_latency_ms = if s.sample_count == 0 {
            latency_ms
        } else {
            s.ema_latency_ms * 0.95 + latency_ms * 0.05
        };
        s.sample_count += 1;

        if s.sample_count % 100 == 0 {
            s.p99_latency_ms = self.recent_max_ms.swap(0, Ordering::Relaxed) as f64;
            self.retune(&mut s);
            s.last_updated_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let _ = self.persist(&s);
            let _ = self.regenerate_configs(&s);
            drop(s); // export_brain が再度 lock するため先に解放
            let _ = self.export_brain(self.brain_path());
            return;
        }
    }

    /// 自己学習の中核ルール:
    ///   - EMAレイテンシが閾値超 → ワーカースレッド増加 (輻輳対策)
    ///   - 十分低い状態が続く → 徐々に減らしメモリ・CPUを節約 (負荷軽減)
    ///   - keepaliveはp99が高いほど短く (詰まった接続の早期解放)
    fn retune(&self, s: &mut TuningState) {
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);

        if s.ema_latency_ms > 200.0 && s.worker_threads < cores * 4 {
            s.worker_threads += 1;
            tracing::info!(new = s.worker_threads, "autoconf: increasing worker_threads (latency high)");
        } else if s.ema_latency_ms < 20.0 && s.worker_threads > cores {
            s.worker_threads -= 1;
            tracing::info!(new = s.worker_threads, "autoconf: decreasing worker_threads (load light, saving resources)");
        }

        s.keepalive_secs = if s.p99_latency_ms > 500.0 { 5 } else { 15 };
    }

    fn persist(&self, s: &TuningState) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(s)?)
    }

    /// 学習結果から Linux 用設定ファイル一式を自動生成する。
    /// 手書き不要 — 生成先は呼び出し側で /etc/systemd/system 等に配置。
    fn regenerate_configs(&self, s: &TuningState) -> std::io::Result<()> {
        let out_dir = self.path.parent().unwrap_or(Path::new(".")).join("generated");
        std::fs::create_dir_all(&out_dir)?;
        std::fs::write(out_dir.join("open-web-server.service"), systemd_unit(s))?;
        std::fs::write(out_dir.join("routes.toml"), default_routes_toml())?;
        Ok(())
    }

    pub fn snapshot(&self) -> TuningState {
        self.state.lock().unwrap().clone()
    }

    /// 「賢さの引き継ぎ」用ポータブルスナップショットを書き出す。
    /// rclone等でGoogleドライブ等へ同期する対象はこのファイル1つでよい。
    /// BLAKE3チェックサムを同梱し、同期経路での破損(途中で切れた等)を検出可能にする。
    pub fn export_brain(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let s = self.state.lock().unwrap().clone();
        let brain = BrainSnapshot {
            format_version: 1,
            exported_unix: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            state: s,
        };
        let body = serde_json::to_vec_pretty(&brain)?;
        let checksum = blake3::hash(&body).to_hex().to_string();
        let wrapper = serde_json::json!({ "checksum_blake3": checksum, "brain": brain });
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(&wrapper)?)
    }

    /// 引っ越し先(新PC・OS再インストール後)で呼ぶ。チェックサム検証後に学習状態を復元する。
    /// 検証に失敗した場合は「新規学習」に安全側フォールバックする(誤った値の適用を防ぐ)。
    pub fn import_brain(&self, path: impl AsRef<Path>) -> std::io::Result<bool> {
        let raw = std::fs::read(&path)?;
        let wrapper: serde_json::Value = serde_json::from_slice(&raw)?;
        let (Some(claimed), Some(brain_val)) =
            (wrapper.get("checksum_blake3").and_then(|v| v.as_str()), wrapper.get("brain"))
        else {
            tracing::warn!("brain file malformed — starting fresh instead of trusting it");
            return Ok(false);
        };
        let recomputed = blake3::hash(&serde_json::to_vec_pretty(brain_val)?).to_hex().to_string();
        if recomputed != claimed {
            tracing::warn!("brain file checksum mismatch (corrupted during sync?) — starting fresh");
            return Ok(false);
        }
        let brain: BrainSnapshot = serde_json::from_value(brain_val.clone())?;
        *self.state.lock().unwrap() = brain.state;
        tracing::info!(from = brain.exported_unix, "restored learned tuning state from portable brain file");
        Ok(true)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BrainSnapshot {
    format_version: u16,
    exported_unix: u64,
    state: TuningState,
}

/// systemd unit を学習済みパラメータで生成する。
/// (Apache/Nginxで言う httpd.conf の Worker/KeepAlive 相当を、環境変数経由で注入)
fn systemd_unit(s: &TuningState) -> String {
    format!(
        r#"# 自動生成: open-web-server autoconf (契約不要のローカル自己学習)
# 生成時刻(unix): {ts}
# 観測統計: EMAレイテンシ={ema:.1}ms, p99={p99:.1}ms, サンプル数={n}
# このファイルは手動編集せず、tuning.json の学習結果に委ねてください。
#
# 対応ディストリビューション: Ubuntu/Debian系 および RHEL/AlmaLinux/Rocky Linux系。
# RHEL系でSELinuxが有効な場合、事前に scripts/install-deps.sh を実行して
# /usr/local/bin/open-web-server へ bin_t コンテキストを付与しておくこと。
# firewalld環境では 8080/tcp, 50051/tcp, 4433/udp の開放も同スクリプトが行う。
[Unit]
Description=open-web-server (open-aruaru Layer 1)
After=network.target open-runo.service

[Service]
Type=simple
Environment=RUNO_GRPC_ADDR=http://127.0.0.1:50051
Environment=OWS_WORKER_THREADS={workers}
Environment=OWS_KEEPALIVE_SECS={keepalive}
ExecStart=/usr/local/bin/open-web-server
Restart=always
RestartSec=2
# ミッションクリティカル運用のためOOM優先度を下げる
OOMScoreAdjust=-500
LimitNOFILE=65536
# RHEL系でSELinuxがサービス起動を拒否する場合は `journalctl -xe` と
# `ausearch -m avc -ts recent` で拒否ログを確認すること。

[Install]
WantedBy=multi-user.target
"#,
        ts = s.last_updated_unix,
        ema = s.ema_latency_ms,
        p99 = s.p99_latency_ms,
        n = s.sample_count,
        workers = s.worker_threads,
        keepalive = s.keepalive_secs,
    )
}

fn default_routes_toml() -> String {
    r#"# 自動生成: open-web-server routes.toml
# proxy_mode により Apache/Nginx 同様、他言語バックエンドを前段に置ける。
# 追記例:
#
# [[routes]]
# path_prefix = "/legacy-php/"
# kind = { type = "Proxy", upstream = "http://127.0.0.1:9000" }
#
# [[routes]]
# path_prefix = "/assets/"
# kind = { type = "Static", root = "/var/www/open-aruaru/assets" }
#
# [[routes]]
# path_prefix = "/"
# kind = { type = "Native" }
"#
    .to_string()
}
