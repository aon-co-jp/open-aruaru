use anyhow::Result;
use poem::{listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};
use tracing::info;

mod agent;
mod api;
mod tasks;
mod prompt;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();

    let port: u16 = std::env::var("ARUARU_AI_PORT")
        .unwrap_or_else(|_| "8082".into()).parse().unwrap_or(8082);

    info!("🤖 aruaru-ai-core 起動 → http://0.0.0.0:{}", port);

    let app = Route::new()
        .nest("/api/ai", api::ai_routes())
        .with(Tracing);

    Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
        .run(app).await?;
    Ok(())
}
