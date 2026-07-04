//! open-client — open-aruaru 統合フロントエンド (Tauri v2 シェル)
//!
//! Electron ではなく Tauri を採用した理由:
//!   - バンドルに Chromium を同梱しない → OSネイティブWebView利用でサイズ 1/10以下、常駐メモリ大幅減
//!   - Rustコアが open-web-server/open-runo/aruaru-db と同じ言語 → 型・ロジック共有が容易
//!   - IPCはプロセス内関数呼び出しに近く、Node.js常駐プロセスが無い分アタック面が小さい
//!   - 予測キャッシュ判断(predictive.rs)をネイティブ層で実行でき、サーバー往復を確実に削減
//!
//! 三重コミットの完全性はここでは絶対にバイパスしない。
//! predictive.rs はあくまで「読み取りの前捌き」用であり、書き込みは常に
//! open-runo への確定経路 (QUIC) を通す。

mod predictive;
mod runo_link;

use predictive::{Decision, HeuristicPredictor, Predictor};
use runo_link::{Commit, CommitOutcome, RunoLink};
use std::sync::Mutex;
use tauri::State;

struct AppState {
    link: RunoLink,
    predictor: Mutex<HeuristicPredictor>,
    /// 予測キャッシュ本体 (key → 最終確定値)。TTL判断は predictor 側が行う。
    cache: Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

/// 書き込み。常にサーバー確定経路 (三重コミット) を通す — バイパス不可。
#[tauri::command]
async fn commit_write(
    state: State<'_, AppState>,
    key: String,
    payload: Vec<u8>,
    finance: bool,
) -> Result<CommitOutcome, String> {
    let priority = if finance { aruaru_common::Priority::Finance } else { aruaru_common::Priority::Game };
    let commit = aruaru_common::Commit::new(key.clone(), payload.clone(), priority);
    let outcome = state.link.commit(commit).await.map_err(|e| e.to_string())?;

    if outcome.committed_layers >= 2 {
        state.cache.lock().unwrap().insert(key, payload);
    }
    Ok(outcome)
}

/// 読み取り。predictive エンジンがローカル/サーバーを選択する。
#[tauri::command]
async fn quorum_read(
    state: State<'_, AppState>,
    key: String,
    finance: bool,
) -> Result<Vec<u8>, String> {
    let decision = state.predictor.lock().unwrap().should_serve_local(&key, finance);

    if decision == Decision::ServeLocal {
        if let Some(v) = state.cache.lock().unwrap().get(&key).cloned() {
            return Ok(v);
        }
        // キャッシュ空振り: サーバーへフォールバックし、ミスとして記録
        state.predictor.lock().unwrap().observe(&key, false);
    }

    // TODO: RunoLink に QuorumRead (gRPC/QUIC) を実装後に接続。
    // 現状は commit_write でキャッシュされた値を確定値として返すスタブ経路。
    state
        .cache
        .lock()
        .unwrap()
        .get(&key)
        .cloned()
        .ok_or_else(|| "not found (quorum read RPC pending)".to_string())
}

fn main() {
    tracing_subscriber::fmt().init();

    // open-runo の QUIC 受け口。本番は設定ファイル/環境変数からエンドポイントを注入する。
    let runo_addr = std::env::var("RUNO_QUIC_ADDR").unwrap_or("127.0.0.1:4433".into());
    let link = RunoLink::connect_insecure(
        runo_addr.parse().expect("invalid RUNO_QUIC_ADDR"),
        "open-runo.local",
    )
    .expect("failed to init QUIC client");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            link,
            predictor: Mutex::new(HeuristicPredictor::default()),
            cache: Mutex::new(std::collections::HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![commit_write, quorum_read])
        .run(tauri::generate_context!())
        .expect("error while running open-client");
}
