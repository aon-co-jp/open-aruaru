use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
pub type Pool = SqlitePool;

pub async fn init_pool(url: &str) -> Result<Pool> {
    if let Some(path) = url.strip_prefix("sqlite:") {
        if let Some(p) = std::path::Path::new(path).parent() {
            tokio::fs::create_dir_all(p).await?;
        }
    }
    let pool = SqlitePoolOptions::new().max_connections(5).connect(url).await?;
    sqlx::migrate!("./src/db/migrations").run(&pool).await?;
    Ok(pool)
}
