use chrono::Utc;

use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct PlatformDomain {
    pub name: String,
    pub is_primary: i64,
    pub https_enabled: i64,
    pub auto_https: i64,
    pub last_checked: Option<String>,
    pub created_at: String,
}

pub fn normalize(name: &str) -> anyhow::Result<String> {
    let mut s = name.trim().to_lowercase();
    for prefix in ["https://", "http://"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }
    s = s.trim_end_matches('/').to_string();
    if s.is_empty() || !s.contains('.') {
        anyhow::bail!("invalid domain");
    }
    Ok(s)
}

impl AppState {
    pub async fn list_platform_domains(&self) -> anyhow::Result<Vec<PlatformDomain>> {
        Ok(sqlx::query_as::<_, PlatformDomain>(
            "SELECT name, is_primary, https_enabled, auto_https, last_checked, created_at
             FROM platform_domains ORDER BY is_primary DESC, created_at",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn primary_platform_domain(&self) -> anyhow::Result<Option<PlatformDomain>> {
        Ok(sqlx::query_as::<_, PlatformDomain>(
            "SELECT name, is_primary, https_enabled, auto_https, last_checked, created_at
             FROM platform_domains WHERE is_primary = 1 LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub fn effective_public_url(&self) -> String {
        self.read_config_file()
            .map(|c| c.server.public_url)
            .unwrap_or_else(|_| self.config.server.public_url.clone())
    }

    pub async fn add_platform_domain(&self, name: &str) -> anyhow::Result<PlatformDomain> {
        let name = normalize(name)?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains")
            .fetch_one(&self.pool)
            .await?;
        let is_primary = if count == 0 { 1 } else { 0 };

        sqlx::query(
            "INSERT INTO platform_domains (name, is_primary, https_enabled, auto_https, last_checked, created_at)
             VALUES ($1, $2, 0, 1, NULL, $3)",
        )
        .bind(&name)
        .bind(is_primary)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        if is_primary == 1 {
            self.apply_primary(&name, false).await?;
        }

        self.write_platform_vhost()?;
        self.get_platform_domain(&name).await
    }

    /// HTTPS 有効化後 — 主ドメインの public_url を https:// に更新
    pub async fn on_https_enabled(&self, domain: &str) -> anyhow::Result<()> {
        let name = normalize(domain)?;
        let is_primary: i64 = sqlx::query_scalar(
            "SELECT is_primary FROM platform_domains WHERE name = $1",
        )
        .bind(&name)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(0);
        if is_primary != 0 {
            self.apply_primary(&name, true).await?;
        }
        let _ = crate::vhost::regenerate_all(self).await;
        self.write_platform_vhost()?;
        Ok(())
    }

    pub async fn delete_platform_domain(&self, name: &str) -> anyhow::Result<()> {
        let name = normalize(name)?;
        let row = self.get_platform_domain(&name).await?;
        if row.is_primary != 0 {
            let others: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM platform_domains WHERE name != $1 ORDER BY created_at LIMIT 1",
            )
            .bind(&name)
            .fetch_all(&self.pool)
            .await?;
            if let Some(next) = others.first() {
                self.set_primary_platform_domain(next).await?;
            } else {
                self.set_public_url(format!(
                    "http://127.0.0.1:{}",
                    self.config.server.port
                ))
                .await?;
            }
        }

        sqlx::query("DELETE FROM platform_domains WHERE name = $1")
            .bind(&name)
            .execute(&self.pool)
            .await?;

        self.write_platform_vhost()?;
        Ok(())
    }

    pub async fn set_primary_platform_domain(&self, name: &str) -> anyhow::Result<()> {
        let name = normalize(name)?;
        let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains WHERE name = $1")
            .bind(&name)
            .fetch_one(&self.pool)
            .await?;
        if exists == 0 {
            anyhow::bail!("domain not found");
        }

        sqlx::query("UPDATE platform_domains SET is_primary = 0")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE platform_domains SET is_primary = 1 WHERE name = $1")
            .bind(&name)
            .execute(&self.pool)
            .await?;

        let https: i64 = sqlx::query_scalar(
            "SELECT https_enabled FROM platform_domains WHERE name = $1",
        )
        .bind(&name)
        .fetch_one(&self.pool)
        .await?;

        self.apply_primary(&name, https != 0).await?;
        self.write_platform_vhost()?;
        Ok(())
    }

    pub async fn swap_platform_domain(&self, from: &str, to: &str) -> anyhow::Result<()> {
        let from = normalize(from)?;
        let to = normalize(to)?;
        if from == to {
            anyhow::bail!("same domain");
        }
        let from_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains WHERE name = $1")
            .bind(&from)
            .fetch_one(&self.pool)
            .await?;
        let to_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains WHERE name = $1")
            .bind(&to)
            .fetch_one(&self.pool)
            .await?;
        if from_exists == 0 || to_exists == 0 {
            anyhow::bail!("domain not found");
        }

        sqlx::query("UPDATE platform_domains SET is_primary = 0 WHERE name = $1")
            .bind(&from)
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE platform_domains SET is_primary = 1 WHERE name = $1")
            .bind(&to)
            .execute(&self.pool)
            .await?;

        let https: i64 = sqlx::query_scalar(
            "SELECT https_enabled FROM platform_domains WHERE name = $1",
        )
        .bind(&to)
        .fetch_one(&self.pool)
        .await?;

        self.apply_primary(&to, https != 0).await?;
        self.write_platform_vhost()?;
        Ok(())
    }

    async fn apply_primary(&self, name: &str, https: bool) -> anyhow::Result<()> {
        let scheme = if https { "https" } else { "http" };
        self.set_public_url(format!("{scheme}://{name}")).await
    }

    pub async fn save_config(&self) -> anyhow::Result<()> {
        let cfg = self.read_config_file()?;
        self.write_config_file(&cfg).await
    }

    pub fn read_config_file(&self) -> anyhow::Result<crate::config::AppConfig> {
        let path = self.data_dir.join("config.toml");
        if path.exists() {
            Ok(toml::from_str(&std::fs::read_to_string(&path)?)?)
        } else {
            Ok(self.config.clone())
        }
    }

    pub async fn write_config_file_pub(&self, cfg: &crate::config::AppConfig) -> anyhow::Result<()> {
        self.write_config_file(cfg).await
    }

    async fn write_config_file(&self, cfg: &crate::config::AppConfig) -> anyhow::Result<()> {
        std::fs::write(
            self.data_dir.join("config.toml"),
            toml::to_string_pretty(cfg)?,
        )?;
        Ok(())
    }

    async fn set_public_url(&self, url: String) -> anyhow::Result<()> {
        let mut cfg = self.read_config_file()?;
        cfg.server.public_url = url;
        self.write_config_file(&cfg).await
    }

    async fn get_platform_domain(&self, name: &str) -> anyhow::Result<PlatformDomain> {
        sqlx::query_as::<_, PlatformDomain>(
            "SELECT name, is_primary, https_enabled, auto_https, last_checked, created_at
             FROM platform_domains WHERE name = $1",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub fn write_platform_vhost(&self) -> anyhow::Result<()> {
        let dir = self.data_dir.join("platform");
        std::fs::create_dir_all(&dir)?;
        std::fs::write(
            dir.join("README.txt"),
            "# Platform domains — vhost configs in data/vhost/\nRun feature: vhost.regenerate\n",
        )?;
        Ok(())
    }

    pub async fn list_all_https_targets(&self) -> anyhow::Result<Vec<HttpsTarget>> {
        let mut out = Vec::new();
        for d in self.list_platform_domains().await? {
            out.push(HttpsTarget {
                name: d.name,
                auto_https: d.auto_https,
                table: "platform".into(),
            });
        }
        for d in self.list_domains().await? {
            out.push(HttpsTarget {
                name: d.name,
                auto_https: d.auto_https,
                table: "site".into(),
            });
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct HttpsTarget {
    pub name: String,
    pub auto_https: i64,
    pub table: String,
}
