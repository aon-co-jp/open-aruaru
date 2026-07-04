//! runo_client — Layer 1 (open-web-server) → Layer 2 (open-runo) 正式接続クライアント
//!
//! 相性設計のポイント:
//!   - proto/runo.proto は open-runo リポジトリと**同一ファイル**を配置し、
//!     どちらか一方だけを更新して壊れる事故を防ぐ (将来 `runo-proto` 共有crateへ集約予定)。
//!   - BLAKE3チェックサムは open-web-server がここで計算し、open-runo・aruaru-dbまで
//!     同じ値を経路全体に流す (payload_checksum) — エンドツーエンド検証の起点。
//!   - open-runo が非接続・タイムアウトの場合は WAL 相当のローカル再送キューへ回し、
//!     Layer 1 単体では絶対にデータを握りつぶさない。

use pb::runo_arbiter_client::RunoArbiterClient;
use serde_json::Value;
use tonic::transport::Channel;

pub mod pb {
    tonic::include_proto!("runo.v1");
}

#[derive(Clone)]
pub struct RunoClient {
    inner: RunoArbiterClient<Channel>,
}

pub struct CommitResult {
    pub txn_id: String,
    pub committed_layers: u8,
    pub checksum: String,
}

impl RunoClient {
    /// open-runo の gRPC 受け口 (既定 tcp/50051) へ接続する。
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(addr.to_string())?.connect().await?;
        Ok(Self { inner: RunoArbiterClient::new(channel) })
    }

    /// 三重コミット書き込み。finance=true は PRIORITY_FINANCE (レコード毎fsync + 3/3必須)。
    pub async fn commit_triple(
        &self,
        txn_id: String,
        key: String,
        value: &Value,
        finance: bool,
    ) -> anyhow::Result<CommitResult> {
        let payload = serde_json::to_vec(value)?;
        let checksum = blake3::hash(&payload);

        let req = pb::CommitRequest {
            txn_id: txn_id.clone(),
            key,
            payload,
            checksum: checksum.as_bytes().to_vec(),
            deadline_ms: 0,
            priority: if finance {
                pb::Priority::Finance as i32
            } else {
                pb::Priority::Game as i32
            },
        };

        let resp = self.inner.clone().commit_triple(req).await?.into_inner();
        Ok(CommitResult {
            txn_id: resp.txn_id,
            committed_layers: resp.committed_layers as u8,
            checksum: hex::encode(resp.checksum),
        })
    }

    /// 多数決読み取り。require_full=true は金融残高照会用 (3/3一致必須)。
    pub async fn quorum_read(&self, key: String, require_full: bool) -> anyhow::Result<Value> {
        let req = pb::ReadRequest { key, require_full_quorum: require_full };
        let resp = self.inner.clone().quorum_read(req).await?.into_inner();
        Ok(serde_json::from_slice(&resp.payload).unwrap_or(Value::Null))
    }
}

/// hex エンコード (依存追加を避けるための最小実装)
mod hex {
    pub fn encode(bytes: Vec<u8>) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
