//! predictive — open-client 独自の「予測と選択判断」エンジン。
//!
//! 目的: サーバー(open-runo/aruaru-db)への往復を減らし、体感速度を上げつつ
//! サーバー負荷を下げる。3層の完全性保証(三重コミット)は一切バイパスしない
//! ——本モジュールは「読み取りの前捌き」だけを担当し、書き込みは常にサーバーへ通す。
//!
//! アルゴリズム (v1, 軽量統計モデル。将来 aruaru-llm 連携へ差し替え可能な trait 設計):
//!   - 各 key ごとに アクセス頻度(EMA) と 最終更新時刻 を追跡
//!   - hot かつ finance=false (GAMEアイテム等) → ローカルキャッシュを signed TTL 付きで許可
//!   - finance=true (残高等) → 常にサーバーへ quorum read (キャッシュ絶対不可)
//!   - 予測ミス率が閾値を超えたキーは自動的に "cold" 扱いへ降格 (安全側に倒す)

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 将来 aruaru-llm 等の実モデルに差し替えるためのインターフェース。
pub trait Predictor: Send + Sync {
    /// このキーをローカルキャッシュから返してよいか判定する。
    fn should_serve_local(&mut self, key: &str, finance: bool) -> Decision;
    /// サーバーから確定値を受け取った際に統計を更新する。
    fn observe(&mut self, key: &str, hit_was_correct: bool);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// ローカルキャッシュ値をそのまま返してよい (サーバー往復を省略)
    ServeLocal,
    /// サーバーへ quorum read を投げる (通常経路)
    FetchServer,
}

struct KeyStat {
    ema_freq: f64,
    last_access: Instant,
    miss_streak: u8,
}

/// v1: EMAベースの軽量ヒューリスティック予測器。
pub struct HeuristicPredictor {
    stats: HashMap<String, KeyStat>,
    /// この頻度(アクセス/秒相当のEMA)を超えたら "hot" とみなす
    hot_threshold: f64,
    /// hot キーのローカルキャッシュ許容 TTL
    cache_ttl: Duration,
}

impl Default for HeuristicPredictor {
    fn default() -> Self {
        Self {
            stats: HashMap::new(),
            hot_threshold: 0.5,
            cache_ttl: Duration::from_millis(800),
        }
    }
}

impl Predictor for HeuristicPredictor {
    fn should_serve_local(&mut self, key: &str, finance: bool) -> Decision {
        // 金融データは予測対象外。常に確定経路 (quorum read) へ。
        if finance {
            return Decision::FetchServer;
        }

        let now = Instant::now();
        let stat = self.stats.entry(key.to_string()).or_insert(KeyStat {
            ema_freq: 0.0,
            last_access: now,
            miss_streak: 0,
        });

        let dt = now.duration_since(stat.last_access).as_secs_f64().max(0.001);
        // EMA更新: 直近アクセス間隔が短いほど頻度スコアが上がる
        let instant_freq = 1.0 / dt;
        stat.ema_freq = stat.ema_freq * 0.7 + instant_freq * 0.3;
        stat.last_access = now;

        let within_ttl = dt < self.cache_ttl.as_secs_f64();
        let is_hot = stat.ema_freq >= self.hot_threshold;
        let trusted = stat.miss_streak < 3; // 予測ミスが続いたら降格

        if is_hot && within_ttl && trusted {
            Decision::ServeLocal
        } else {
            Decision::FetchServer
        }
    }

    fn observe(&mut self, key: &str, hit_was_correct: bool) {
        if let Some(stat) = self.stats.get_mut(key) {
            if hit_was_correct {
                stat.miss_streak = 0;
            } else {
                stat.miss_streak = stat.miss_streak.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finance_never_cached() {
        let mut p = HeuristicPredictor::default();
        assert_eq!(p.should_serve_local("acct:1:balance", true), Decision::FetchServer);
    }

    #[test]
    fn cold_key_fetches_server() {
        let mut p = HeuristicPredictor::default();
        assert_eq!(p.should_serve_local("item:sword", false), Decision::FetchServer);
    }

    #[test]
    fn repeated_miss_demotes_to_server() {
        let mut p = HeuristicPredictor::default();
        p.should_serve_local("item:sword", false);
        for _ in 0..3 {
            p.observe("item:sword", false);
        }
        // hot になっていても miss_streak >= 3 なら降格
        p.stats_insert_hot_for_test();
        assert_eq!(p.should_serve_local("item:sword", false), Decision::FetchServer);
    }
}

#[cfg(test)]
impl HeuristicPredictor {
    fn stats_insert_hot_for_test(&mut self) {
        if let Some(s) = self.stats.get_mut("item:sword") {
            s.ema_freq = 10.0;
            s.last_access = std::time::Instant::now();
        }
    }
}
