use std::fs;
use std::path::Path;

use chrono::Utc;
use sqlx::AnyPool;

use super::manifest::{load, path, PluginManifest};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct InstalledPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    pub installed_at: String,
}

pub async fn list_installed(pool: &AnyPool) -> anyhow::Result<Vec<InstalledPlugin>> {
    Ok(sqlx::query_as::<_, InstalledPlugin>(
        "SELECT id, name, version, kind, installed_at FROM installed_plugins ORDER BY installed_at",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn install(
    pool: &AnyPool,
    source: &Path,
    dest_root: &Path,
    target: &str,
) -> anyhow::Result<Vec<String>> {
    let mf = load(&path(source, target))?;
    let mut order = Vec::new();
    if mf.is_bundle() {
        for id in &mf.plugins {
            collect_order(source, id, &mut order)?;
        }
    } else {
        collect_order(source, &mf.id, &mut order)?;
    }

    for id in &order {
        install_one(pool, source, dest_root, id).await?;
    }
    if mf.is_bundle() {
        register(pool, &mf).await?;
    }

    let mut out = order;
    if mf.is_bundle() {
        out.push(mf.id);
    }
    Ok(out)
}

fn collect_order(source: &Path, id: &str, out: &mut Vec<String>) -> anyhow::Result<()> {
    if out.iter().any(|x| x == id) {
        return Ok(());
    }
    let m = load(&path(source, id))?;
    for dep in &m.requires {
        collect_order(source, dep, out)?;
    }
    out.push(id.to_string());
    Ok(())
}

async fn install_one(pool: &AnyPool, source: &Path, dest_root: &Path, id: &str) -> anyhow::Result<()> {
    let m = load(&path(source, id))?;
    let src = source.join(id);
    let dest = dest_root.join(id);
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    copy_tree(&src, &dest)?;
    register(pool, &m).await
}

fn copy_tree(src: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let from = e.path();
        let to = dest.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

async fn register(pool: &AnyPool, m: &PluginManifest) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO installed_plugins (id, name, version, kind, installed_at) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT(id) DO UPDATE SET version=excluded.version, installed_at=excluded.installed_at",
    )
    .bind(&m.id)
    .bind(&m.name)
    .bind(&m.version)
    .bind(if m.is_bundle() { "bundle" } else { "plugin" })
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}
