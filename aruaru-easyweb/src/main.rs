use anyhow::Result;
use poem::{
    get, post,
    listener::TcpListener,
    middleware::Tracing,
    EndpointExt, Route, Server,
};
use tracing::info;

mod auth;
mod config;
mod db;
mod distro;
mod dns;
mod domain;
mod firewall;
mod routes;
mod scheduler;
mod settings;
mod webserver;

use aruaru_ai_core::routes as ai;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("aruaru_easyweb=info".parse()?),
        )
        .init();

    let cfg = config::AppConfig::load()?;
    let pool = db::init_pool(&cfg.database_url).await?;

    let live_cfg = if settings::is_setup_completed(&pool).await {
        let s = settings::load(&pool).await?;
        s.to_app_config()
    } else {
        cfg.clone()
    };

    scheduler::start(pool.clone()).await?;

    // Linux ディストリビューション検出 & ファイアウォール自動開放
    let sys = distro::detect().await;
    info!("🐧 OS: {} {}", sys.distro_name(), sys.version);
    firewall::open_port(live_cfg.port, &sys).await;

    info!("🌐 aruaru-easyweb 起動 → http://{}:{}", live_cfg.host, live_cfg.port);
    if !settings::is_setup_completed(&pool).await {
        info!("⚠️  初回セットアップが必要です → http://{}:{}/setup", live_cfg.host, live_cfg.port);
    }

    let app = Route::new()
        // ルートレベルのページ（nest("/") 重複を避けるため直接 .at() で登録）
        .at("/setup",    get(routes::setup::setup_page).post(routes::setup::setup_save))
        .at("/settings", get(routes::setup::settings_page).post(routes::setup::settings_save))
        .at("/",         get(routes::ui::index))
        .at("/login",    get(routes::ui::login_page))
        .at("/anthropic",            get(ai::anthropic_page).post(ai::connect))
        .at("/anthropic/disconnect", post(ai::disconnect))
        // API
        .at("/api/detect-ip", get(routes::setup::detect_ip))
        .nest("/api/auth",      routes::auth::auth_routes(pool.clone(), live_cfg.clone()))
        .nest("/api/domain",    routes::domain::domain_routes(pool.clone(), live_cfg.clone()))
        .nest("/api/webserver", routes::webserver::webserver_routes(pool.clone()))
        .nest("/api/dns",       routes::dns::dns_api_routes(pool.clone()))
        .at("/dns",             get(routes::dns::dns_page))
        // 静的ファイル
        .nest("/static", poem::endpoint::StaticFilesEndpoint::new("./static"))
        .data(pool.clone())
        .with(Tracing);

    Server::new(TcpListener::bind(format!("{}:{}", live_cfg.host, live_cfg.port)))
        .run(app)
        .await?;

    Ok(())
}
