use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfig {
    pub host:         String,
    pub port:         u16,
    pub database_url: String,
    pub ai_core_url:  String,   // aruaru-ai-core の URL
    pub easyweb_url:  String,   // aruaru-easyweb の URL
}

impl CoreConfig {
    pub fn load() -> Result<Self> {
        let c = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("ARUARU_CORE").separator("__"))
            .set_default("host",         "0.0.0.0")?
            .set_default("port",         8081)?
            .set_default("database_url", "sqlite:data/core.db")?
            .set_default("ai_core_url",  "http://localhost:8082")?
            .set_default("easyweb_url",  "http://localhost:8080")?
            .build()?;
        Ok(c.try_deserialize()?)
    }
}
