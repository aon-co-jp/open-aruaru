//! HTTP エントリ。ルーティングと状態の束ねだけにし、処理は `api` に集約する。

mod api;

use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let data_dir = env::var("RENTAL_SYNC_DATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("./data"));

    let state = Arc::new(RwLock::new(
        api::AppState::load_or_default(&data_dir).expect("data dir init"),
    ));

    let app = Router::new()
        .route("/", get(api::root))
        .route("/api/health", get(api::health))
        .route("/api/servers", get(api::list_servers).post(api::create_server))
        .route(
            "/api/servers/:id",
            get(api::get_server).delete(api::delete_server),
        )
        .route("/api/backup", post(api::run_backup))
        .route("/api/config/backup-paths", get(api::get_backup_paths).post(api::set_backup_paths))
        .route("/api/git/check", get(api::git_check))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let host = env::var("RENTAL_SYNC_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = env::var("RENTAL_SYNC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3847);
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("bind address");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind");
    eprintln!("rental-sync-hub listening on http://{}", addr);
    axum::serve(listener, app).await.expect("serve");
}
