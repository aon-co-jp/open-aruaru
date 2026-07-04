use std::sync::Arc;

use poem::{
    endpoint::StaticFilesEndpoint,
    listener::TcpListener,
    middleware::{Compression, Cors, Tracing},
    EndpointExt, Server,
};
use poem::session::{CookieConfig, CookieSession};

use crate::github_crawler;
use crate::https_monitor;
use crate::routes;
use crate::state::{AppState, SharedState};

pub async fn serve(data_dir: &std::path::Path, root_dir: &std::path::Path) -> anyhow::Result<()> {
    let state = AppState::new(data_dir, root_dir).await?;
    let shared: SharedState = Arc::new(state);

    https_monitor::spawn(shared.clone());
    // GitHub 自動クロール（10分ごとに最新データ取得）
    github_crawler::spawn_github_crawler(shared.clone());

    let host = shared.config.server.host.clone();
    let port = shared.config.server.port;

    let session = CookieSession::new(CookieConfig::new().http_only(true));
    let static_files = StaticFilesEndpoint::new(root_dir.join("static"));

    let app = routes::routes()
        .nest("/static", static_files)
        .data(shared)
        .with(session)
        .with(Tracing)
        .with(Compression::new())
        .with(Cors::new());

    tracing::info!("aruaru-easyweb v0.1 on http://{host}:{port}/");
    tracing::info!("register: http://{host}:{port}/register");

    Server::new(TcpListener::bind(format!("{host}:{port}")))
        .run(app)
        .await?;

    Ok(())
}
