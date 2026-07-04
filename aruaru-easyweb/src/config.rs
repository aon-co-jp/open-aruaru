use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host:                  String,
    pub port:                  u16,
    pub database_url:          String,
    pub app_url:               String,
    pub token_expiry_minutes:  i64,
    pub admin_email:           String,
    pub smtp:                  SmtpConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host:     String,
    pub port:     u16,
    pub username: String,
    pub password: String,
    pub from:     String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let c = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("ARUARU").separator("__"))
            .set_default("host",                 "0.0.0.0")?
            .set_default("port",                 8080)?
            .set_default("database_url",         "sqlite:data/aruaru.db")?
            .set_default("app_url",              "http://localhost:8080")?
            .set_default("token_expiry_minutes", 15)?
            .set_default("admin_email",          "")?
            .build()?;
        Ok(c.try_deserialize()?)
    }
}
