use anyhow::Result;
use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use tracing::info;

mod config;
mod db;
mod keygen;
mod routes;
mod settings;
mod sftp;
mod ssh;
mod vps;

// AI は含まない — aruaru-ai-core (lib) が別途 aruaru-easyweb に組み込まれる

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("aruaru_core=info".parse()?))
        .init();

    let cfg  = config::CoreConfig::load()?;
    let pool = db::init_pool(&cfg.database_url).await?;

    info!("🔧 aruaru-core 起動 → http://{}:{}", cfg.host, cfg.port);

    let app = Route::new()
        .nest("/",           routes::ui::ui_routes(pool.clone()))
        .nest("/api/vps",    routes::vps::vps_routes(pool.clone()))
        .nest("/api/ssh",    routes::ssh::ssh_routes(pool.clone()))
        .nest("/api/sftp",   routes::sftp::sftp_routes(pool.clone()))
        .nest("/api/keys",   routes::keys::key_routes(pool.clone()))
        .nest("/static",     poem::endpoint::StaticFilesEndpoint::new("./static"))
        .with(Tracing);

    Server::new(TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)))
        .run(app).await?;
    Ok(())
}
