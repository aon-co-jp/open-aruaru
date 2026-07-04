use std::path::Path;

use sqlx::AnyPool;

use super::manifest::{load, path, PluginManifest};

pub async fn uninstall(
    pool: &AnyPool,
    source: &Path,
    dest_root: &Path,
    id: &str,
    _purge_data: bool,
) -> anyhow::Result<()> {
    ensure_not_required_by_others(pool, source, id).await?;

    let dest = dest_root.join(id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }

    sqlx::query("DELETE FROM installed_plugins WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn ensure_not_required_by_others(
    pool: &AnyPool,
    source: &Path,
    target_id: &str,
) -> anyhow::Result<()> {
    let installed = super::install::list_installed(pool).await?;
    for row in installed {
        if row.id == target_id {
            continue;
        }
        let mf = load(&path(source, &row.id))?;
        if mf.requires.iter().any(|r| r == target_id) {
            anyhow::bail!(
                "cannot uninstall {target_id}: required by {}",
                row.id
            );
        }
    }
    Ok(())
}

pub async fn is_installed(pool: &AnyPool, id: &str) -> anyhow::Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(1) FROM installed_plugins WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(n,)| n > 0).unwrap_or(false))
}

pub fn plugin_manifest(source: &Path, id: &str) -> anyhow::Result<PluginManifest> {
    load(&path(source, id))
}
