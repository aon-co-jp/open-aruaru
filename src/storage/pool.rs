use std::path::Path;

use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;

use crate::config::DatabaseConfig;

use super::DbBackend;

pub async fn connect(cfg: &DatabaseConfig, data_dir: &Path) -> anyhow::Result<(AnyPool, DbBackend)> {
    let primary = cfg.primary.trim().to_lowercase();
    let (url, backend) = resolve_url(cfg, data_dir, &primary)?;
    tracing::info!("database: connecting ({backend:?})");

    let pool = AnyPoolOptions::new()
        .max_connections(if backend == DbBackend::Postgres {
            10
        } else {
            5
        })
        .connect(&url)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "database connection failed ({primary}): {e}\n\
                 hint: set [database] primary = \"sqlite\" for local dev, or configure postgres_url / ARUARU_DATABASE_URL"
            )
        })?;

    Ok((pool, backend))
}

fn resolve_url(
    cfg: &DatabaseConfig,
    data_dir: &Path,
    primary: &str,
) -> anyhow::Result<(String, DbBackend)> {
    match primary {
        "sqlite" => {
            let path = data_dir.join("aruaru.db");
            Ok((
                format!("sqlite://{}?mode=rwc", path.display()),
                DbBackend::Sqlite,
            ))
        }
        "postgres" | "postgresql" => {
            let url = std::env::var("ARUARU_DATABASE_URL")
                .unwrap_or_else(|_| cfg.postgres_url.clone());
            if url.is_empty() {
                anyhow::bail!("postgres_url is empty — set [database] postgres_url or ARUARU_DATABASE_URL");
            }
            Ok((url, DbBackend::Postgres))
        }
        "cockroach" | "cockroachdb" => {
            let url = std::env::var("ARUARU_COCKROACH_URL")
                .or_else(|_| std::env::var("ARUARU_DATABASE_URL"))
                .unwrap_or_else(|_| cfg.cockroach_url.clone());
            if url.is_empty() {
                anyhow::bail!(
                    "cockroach_url is empty — set [database] cockroach_url or ARUARU_COCKROACH_URL"
                );
            }
            Ok((url, DbBackend::Cockroach))
        }
        other => anyhow::bail!("unknown database.primary: {other}"),
    }
}
