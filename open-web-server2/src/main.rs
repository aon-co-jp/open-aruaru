//! open-web-server
//! open-aruaru (iLumi) プロジェクトの三層ミッションクリティカルWEBサーバー。
//!
//! 三層構成:
//!   Layer 1: open-web-server  (HTTP/WebSocket フロント, Poem) — 本リポジトリ
//!   Layer 2: open-runo        (トランザクション調停, QUIC+gRPC二重受け口)
//!   Layer 3: aruaru-db        (三重書き込み + BLAKE3整合性検証, 3レプリカ)
//!
//! v0.3 での拡張:
//!   - proxy_mode: Apache/Nginx同様、他言語バックエンドを前段配置できる
//!     互換レイヤー (Native / Proxy / Static の3モード)。
//!   - autoconf: 契約不要のローカル自己学習ループが、実トラフィックの
//!     統計から systemd unit / routes.toml を自動生成・自動再調整する。

use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, post,
    web::{Json, Path}, Endpoint, EndpointExt, Route, Server,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

mod autoconf;
mod proxy_mode;
mod runo_client;
mod triple_commit;

use autoconf::AutoConf;
use runo_client::RunoClient;

#[derive(Debug, Serialize, Deserialize)]
struct WriteRequest {
    key: String,
    value: serde_json::Value,
    #[serde(default)]
    finance: bool,
}

#[derive(Debug, Serialize)]
struct WriteResponse {
    txn_id: String,
    committed_layers: u8,
    checksum: String,
    status: &'static str,
}

#[handler]
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "open-web-server",
        "status": "ok",
        "layers": ["open-web-server", "open-runo", "aruaru-db"],
    }))
}

#[handler]
async fn write(
    client: poem::web::Data<&RunoClient>,
    Json(req): Json<WriteRequest>,
) -> Json<WriteResponse> {
    let result = triple_commit::commit(&client, &req.key, &req.value, req.finance).await;
    Json(WriteResponse {
        txn_id: result.txn_id,
        committed_layers: result.committed_layers,
        checksum: result.checksum,
        status: match result.committed_layers {
            3 => "committed",
            1..=2 => "quorum-safe",
            _ => "failed-runo-unreachable",
        },
    })
}

#[handler]
async fn read(
    client: poem::web::Data<&RunoClient>,
    Path(key): Path<String>,
) -> Json<serde_json::Value> {
    let value = triple_commit::quorum_read(&client, &key, false).await;
    Json(serde_json::json!({ "key": key, "value": value }))
}

/// 全リクエストのレイテンシを autoconf の自己学習ループへ流すミドルウェア。
struct AutoConfMiddleware(Arc<AutoConf>);
impl<E: Endpoint> poem::Middleware<E> for AutoConfMiddleware {
    type Output = AutoConfEndpoint<E>;
    fn transform(&self, ep: E) -> Self::Output {
        AutoConfEndpoint { ep, ac: self.0.clone() }
    }
}
struct AutoConfEndpoint<E> {
    ep: E,
    ac: Arc<AutoConf>,
}
impl<E: Endpoint> Endpoint for AutoConfEndpoint<E> {
    type Output = E::Output;
    async fn call(&self, req: poem::Request) -> poem::Result<Self::Output> {
        let start = Instant::now();
        let result = self.ep.call(req).await;
        self.ac.record(start.elapsed().as_secs_f64() * 1000.0);
        result
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,open_web_server=debug")
        .init();

    // 自己学習オートコンフィグ起動 (外部契約・API通信なし。ローカルファイルのみ)
    let autoconf = Arc::new(AutoConf::load_or_init("./data/tuning.json"));
    tracing::info!("autoconf: loaded (契約不要・ローカル学習) -> ./data/generated/*");

    let runo_addr = std::env::var("RUNO_GRPC_ADDR").unwrap_or("http://127.0.0.1:50051".into());
    let runo_client = RunoClient::connect(&runo_addr).await?;
    tracing::info!(%runo_addr, "connected to open-runo (Layer 2)");

    let app = Route::new()
        .at("/health", get(health))
        .at("/v1/kv/:key", get(read))
        .at("/v1/kv", post(write))
        .data(runo_client)
        .with(Tracing)
        .with(AutoConfMiddleware(autoconf));

    tracing::info!("open-web-server listening on 0.0.0.0:8080");
    Server::new(TcpListener::bind("0.0.0.0:8080")).run(app).await?;
    Ok(())
}
