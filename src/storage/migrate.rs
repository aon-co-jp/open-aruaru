use std::path::{Path, PathBuf};

use chrono::Utc;
use sqlx::AnyPool;

use super::DbBackend;

pub async fn run(pool: &AnyPool, backend: DbBackend, root_dir: &Path) -> anyhow::Result<()> {
    let dir = migrations_dir(root_dir, backend);
    if !dir.is_dir() {
        anyhow::bail!("migrations directory not found: {}", dir.display());
    }

    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "sql"))
        .collect();
    files.sort();

    for path in files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let version = parse_version(&name)?;
        if migration_applied(pool, version).await? {
            tracing::debug!("migration skip: {name}");
            continue;
        }

        tracing::info!("migration apply: {name}");
        let sql = std::fs::read_to_string(&path)?;
        for stmt in split_statements(&sql) {
            sqlx::query(&stmt).execute(pool).await?;
        }
        record_migration(pool, backend, version, &name).await?;
    }

    Ok(())
}

fn migrations_dir(root_dir: &Path, backend: DbBackend) -> PathBuf {
    let sub = match backend {
        DbBackend::Sqlite => "sqlite",
        DbBackend::Postgres | DbBackend::Cockroach => "postgres",
    };
    root_dir.join("migrations").join(sub)
}

fn parse_version(name: &str) -> anyhow::Result<i64> {
    let num: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse()
        .map_err(|_| anyhow::anyhow!("invalid migration filename: {name}"))
}

async fn migration_applied(pool: &AnyPool, version: i64) -> anyhow::Result<bool> {
    if !migrations_table_exists(pool).await {
        return Ok(false);
    }
    let applied: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM schema_migrations WHERE version = $1 LIMIT 1",
    )
    .bind(version)
    .fetch_optional(pool)
    .await?;
    Ok(applied.is_some())
}

async fn migrations_table_exists(pool: &AnyPool) -> bool {
    sqlx::query("SELECT 1 FROM schema_migrations LIMIT 1")
        .fetch_optional(pool)
        .await
        .is_ok()
}

async fn record_migration(
    pool: &AnyPool,
    backend: DbBackend,
    version: i64,
    name: &str,
) -> anyhow::Result<()> {
    match backend {
        DbBackend::Sqlite => {
            sqlx::query(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES ($1, $2, $3)",
            )
            .bind(version)
            .bind(name)
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await?;
        }
        DbBackend::Postgres | DbBackend::Cockroach => {
            sqlx::query(
                "INSERT INTO schema_migrations (version, name) VALUES ($1, $2)",
            )
            .bind(version)
            .bind(name)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

fn split_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.lines().filter(|l| !l.trim_start().starts_with("--")).collect::<Vec<_>>().join("\n"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
