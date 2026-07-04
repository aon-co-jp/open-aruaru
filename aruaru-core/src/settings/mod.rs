// aruaru-core 設定（DB保存、VI不要）
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::db::Pool;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreSettings {
    pub ai_core_url:  String,
    pub easyweb_url:  String,
}

pub async fn load(pool: &Pool) -> Result<CoreSettings> {
    let rows = sqlx::query!("SELECT key, value FROM core_settings")
        .fetch_all(pool).await.unwrap_or_default();
    let mut s = CoreSettings::default();
    for r in rows {
        match r.key.as_str() {
            "ai_core_url"  => s.ai_core_url  = r.value,
            "easyweb_url"  => s.easyweb_url  = r.value,
            _ => {}
        }
    }
    Ok(s)
}
