//! proxy_mode — open-web-server を Apache/Nginx 同様の「前段サーバー」として
//! 使うための互換レイヤー。
//!
//! open-web-server は本来 Poem ネイティブハンドラ(Rust)でロジックを実行するが、
//! 既存のPHP/Node/Python資産を前段に置きたいケースのために以下3モードを提供する:
//!
//!   1. Native  — Poem ハンドラ直実行 (open-e-gov / OpenRedmine / OpenWordPress 用)
//!   2. Proxy   — 背後の任意言語アプリ(127.0.0.1:xxxx等)へHTTPリバースプロキシ
//!   3. Static  — 静的ファイル配信 (SPA等)
//!
//! ルーティングは routes.toml (autoconf が自動生成・自動調整) で切り替える。
//! これにより「言語非依存の前段」という Apache/Nginx の強みを取り込みつつ、
//! Rust資産(open-e-gov等)は Native モードで最速経路を使い分けられる。

use poem::{
    handler,
    http::{StatusCode, Uri},
    web::Data,
    Body, Endpoint, IntoResponse, Request, Response,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteKind {
    /// 例: PHP-FPM/Node/Python等、任意言語のバックエンドへ丸投げ
    Proxy { upstream: String },
    /// 静的ファイルディレクトリ
    Static { root: String },
    /// Poem ネイティブハンドラ (このプロセス内、Rustのみ)
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub path_prefix: String,
    pub kind: RouteKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteTable {
    pub routes: Vec<Route>,
}

impl RouteTable {
    pub fn match_route(&self, path: &str) -> Option<&Route> {
        // 最長一致 (Nginx の location 優先度と同じ考え方)
        self.routes
            .iter()
            .filter(|r| path.starts_with(&r.path_prefix))
            .max_by_key(|r| r.path_prefix.len())
    }
}

/// 任意言語バックエンドへの単純リバースプロキシ。
/// Apache の mod_proxy / Nginx の proxy_pass 相当。
pub async fn proxy_to(upstream: &str, req: Request) -> poem::Result<Response> {
    let path_and_query = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let target: Uri = format!("{}{}", upstream.trim_end_matches('/'), path_and_query)
        .parse()
        .map_err(|_| poem::Error::from_status(StatusCode::BAD_GATEWAY))?;

    let client = reqwest::Client::new();
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| poem::Error::from_status(StatusCode::BAD_GATEWAY))?;

    let resp = client
        .request(method, target.to_string())
        .send()
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::BAD_GATEWAY))?;

    let status = resp.status().as_u16();
    let bytes = resp.bytes().await.unwrap_or_default();
    Ok(Response::builder()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY))
        .body(Body::from(bytes.to_vec())))
}

#[handler]
pub async fn dispatch(table: Data<&RouteTable>, req: Request) -> poem::Result<Response> {
    let path = req.uri().path().to_string();
    match table.match_route(&path) {
        Some(Route { kind: RouteKind::Proxy { upstream }, .. }) => {
            proxy_to(upstream, req).await
        }
        Some(Route { kind: RouteKind::Static { root }, .. }) => {
            serve_static(root, &path).await
        }
        _ => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn serve_static(root: &str, path: &str) -> poem::Result<Response> {
    let safe_path = path.trim_start_matches('/').replace("..", "");
    let full = std::path::Path::new(root).join(safe_path);
    match tokio::fs::read(&full).await {
        Ok(bytes) => Ok(Response::builder().status(StatusCode::OK).body(Body::from(bytes))),
        Err(_) => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}
