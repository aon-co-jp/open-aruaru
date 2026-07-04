//! triple_commit — open-runo (Layer 2) への正式委譲ラッパー
//!
//! v0.2: インメモリ・スタブを廃止し、runo_client 経由で実際の open-runo
//! (QUIC/gRPC 二重受け口) へ三重コミットを委譲する。
//! open-web-server 自身はもはや「3レプリカ」を知らない — その責務は
//! Layer 2/3 に完全に閉じており、Layer 1 は「投げて結果を待つ」だけになる。

use crate::runo_client::RunoClient;
use serde_json::Value;

pub struct CommitResult {
    pub txn_id: String,
    pub committed_layers: u8,
    pub checksum: String,
}

/// open-runo への三重コミット委譲。
pub async fn commit(client: &RunoClient, key: &str, value: &Value, finance: bool) -> CommitResult {
    let txn_id = uuid::Uuid::now_v7().to_string();
    match client.commit_triple(txn_id.clone(), key.to_string(), value, finance).await {
        Ok(r) => CommitResult {
            txn_id: r.txn_id,
            committed_layers: r.committed_layers,
            checksum: r.checksum,
        },
        Err(e) => {
            // open-runo 未接続/タイムアウト: Layer 1 は握りつぶさず 0/3 として返す。
            // 呼び出し元 (main.rs) は WAL 相当のローカル再送キューへ積むこと。
            tracing::error!(%txn_id, error = %e, "open-runo unreachable — commit not durable");
            CommitResult { txn_id, committed_layers: 0, checksum: String::new() }
        }
    }
}

/// open-runo への多数決読み取り委譲。
pub async fn quorum_read(client: &RunoClient, key: &str, require_full: bool) -> Value {
    client.quorum_read(key.to_string(), require_full).await.unwrap_or(Value::Null)
}
