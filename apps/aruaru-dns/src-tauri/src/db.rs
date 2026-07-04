use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::AppConfig;

const MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS dns_records (
    id BIGSERIAL PRIMARY KEY,
    zone TEXT NOT NULL,
    name TEXT NOT NULL,
    record_type TEXT NOT NULL,
    value TEXT NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 3600,
    created_at TEXT NOT NULL,
    UNIQUE (zone, name, record_type, value)
);
CREATE INDEX IF NOT EXISTS idx_dns_records_zone ON dns_records (zone);
"#;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct DnsRecord {
    pub id: i64,
    pub zone: String,
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: i64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct DualDb {
    pub postgres: PgPool,
    pub aruaru_db: Option<PgPool>,
    pub config: AppConfig,
}

impl DualDb {
    pub async fn connect(config: AppConfig) -> anyhow::Result<Self> {
        let postgres = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.postgres.url)
            .await?;

        run_migration(&postgres).await?;

        let aruaru_db = if config.aruaru_db.sync_enabled {
            match PgPoolOptions::new()
                .max_connections(3)
                .connect(&config.aruaru_db.url)
                .await
            {
                Ok(pool) => {
                    let _ = run_migration(&pool).await;
                    Some(pool)
                }
                Err(e) => {
                    eprintln!("aruaru-db connect skipped: {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            postgres,
            aruaru_db,
            config,
        })
    }

    pub fn postgres_ok(&self) -> bool {
        !self.postgres.is_closed()
    }

    pub fn aruaru_db_ok(&self) -> bool {
        self.aruaru_db
            .as_ref()
            .is_some_and(|p| !p.is_closed())
    }
}

pub async fn run_migration(pool: &PgPool) -> anyhow::Result<()> {
    for stmt in MIGRATION.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

pub fn normalize_zone(name: &str) -> anyhow::Result<String> {
    let mut s = name.trim().to_lowercase();
    for prefix in ["https://", "http://"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }
    s = s.trim_end_matches('/').to_string();
    if s.is_empty() || !s.contains('.') {
        anyhow::bail!("invalid zone");
    }
    Ok(s)
}

pub fn normalize_record_name(name: &str) -> anyhow::Result<String> {
    let s = name.trim().to_lowercase();
    if s.is_empty() || s == "@" {
        return Ok("@".into());
    }
    Ok(s.trim_end_matches('.').to_string())
}

const RECORD_TYPES: &[&str] = &["A", "AAAA", "CNAME", "TXT", "NS", "MX"];

pub fn normalize_record_type(t: &str) -> anyhow::Result<String> {
    let u = t.trim().to_uppercase();
    if RECORD_TYPES.contains(&u.as_str()) {
        Ok(u)
    } else {
        anyhow::bail!("unsupported record type");
    }
}

pub async fn list_records(pool: &PgPool, zone: Option<&str>) -> anyhow::Result<Vec<DnsRecord>> {
    if let Some(z) = zone {
        let z = normalize_zone(z)?;
        Ok(sqlx::query_as(
            "SELECT id, zone, name, record_type, value, ttl, created_at
             FROM dns_records WHERE zone = $1 ORDER BY name, record_type",
        )
        .bind(z)
        .fetch_all(pool)
        .await?)
    } else {
        Ok(sqlx::query_as(
            "SELECT id, zone, name, record_type, value, ttl, created_at
             FROM dns_records ORDER BY zone, name, record_type",
        )
        .fetch_all(pool)
        .await?)
    }
}

pub async fn insert_record(
    pool: &PgPool,
    zone: &str,
    name: &str,
    record_type: &str,
    value: &str,
    ttl: i64,
) -> anyhow::Result<DnsRecord> {
    let zone = normalize_zone(zone)?;
    let name = normalize_record_name(name)?;
    let record_type = normalize_record_type(record_type)?;
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!("value required");
    }
    let ttl = if ttl <= 0 { 3600 } else { ttl };
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO dns_records (zone, name, record_type, value, ttl, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (zone, name, record_type, value) DO NOTHING",
    )
    .bind(&zone)
    .bind(&name)
    .bind(&record_type)
    .bind(&value)
    .bind(ttl)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query_as(
        "SELECT id, zone, name, record_type, value, ttl, created_at
         FROM dns_records WHERE zone = $1 AND name = $2 AND record_type = $3 AND value = $4",
    )
    .bind(&zone)
    .bind(&name)
    .bind(&record_type)
    .bind(&value)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn delete_record(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    let n = sqlx::query("DELETE FROM dns_records WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        anyhow::bail!("record not found");
    }
    Ok(())
}

pub async fn sync_all(src: &PgPool, dst: &PgPool) -> anyhow::Result<u64> {
    let rows = list_records(src, None).await?;
    let mut n = 0u64;
    for r in rows {
        insert_record(&dst, &r.zone, &r.name, &r.record_type, &r.value, r.ttl).await?;
        n += 1;
    }
    let _ = sqlx::query("SELECT aruaru_commit($1)")
        .bind(format!("aruaru-dns sync {} records", n))
        .execute(dst)
        .await;
    Ok(n)
}

#[derive(Debug, serde::Serialize)]
pub struct DnsGuide {
    pub zone: String,
    pub vps_ip: String,
    pub nameservers: Vec<String>,
    pub steps: Vec<String>,
    pub suggested_records: Vec<SuggestedRecord>,
}

#[derive(Debug, serde::Serialize)]
pub struct SuggestedRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: i64,
    pub note: String,
}

pub fn build_guide(zone: &str, vps_ip: &str, nameservers: &[String]) -> anyhow::Result<DnsGuide> {
    let zone = normalize_zone(zone)?;
    let ip = vps_ip.trim().to_string();
    if ip.is_empty() {
        anyhow::bail!("vps_ip required");
    }
    let ns = if nameservers.is_empty() {
        vec!["ns1.example.com".into(), "ns2.example.com".into()]
    } else {
        nameservers.to_vec()
    };
    Ok(DnsGuide {
        zone: zone.clone(),
        vps_ip: ip.clone(),
        nameservers: ns,
        steps: vec![
            format!("レジストラで {zone} の DNS 設定を開く"),
            "ネームサーバーを VPS 用に変更".into(),
            format!("A レコード @ → {ip}"),
            format!("サブドメイン panel.{zone} → {ip}（任意）"),
            "aruaru 本体で platform ドメインを登録".into(),
            "aruaru-https で HTTPS 自動化".into(),
        ],
        suggested_records: vec![
            SuggestedRecord {
                record_type: "A".into(),
                name: "@".into(),
                value: ip.clone(),
                ttl: 3600,
                note: "ルート".into(),
            },
            SuggestedRecord {
                record_type: "A".into(),
                name: "www".into(),
                value: ip,
                ttl: 3600,
                note: "www".into(),
            },
        ],
    })
}
