use chrono::Utc;

use crate::platform_domain::normalize;
use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct DnsRecord {
    pub id: i64,
    pub zone: String,
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DnsGuide {
    pub zone: String,
    pub vps_ip: String,
    pub nameservers: Vec<String>,
    pub steps: Vec<String>,
    pub suggested_records: Vec<SuggestedRecord>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SuggestedRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: i64,
    pub note: String,
}

const RECORD_TYPES: &[&str] = &["A", "AAAA", "CNAME", "TXT", "NS", "MX"];

impl AppState {
    pub async fn is_plugin_installed(&self, id: &str) -> bool {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM installed_plugins WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0)
            > 0
    }

    pub async fn list_dns_records(&self, zone: Option<&str>) -> anyhow::Result<Vec<DnsRecord>> {
        if let Some(z) = zone {
            let z = normalize(z)?;
            Ok(sqlx::query_as::<_, DnsRecord>(
                "SELECT id, zone, name, record_type, value, ttl, created_at
                 FROM dns_records WHERE zone = $1 ORDER BY name, record_type",
            )
            .bind(&z)
            .fetch_all(&self.pool)
            .await?)
        } else {
            Ok(sqlx::query_as::<_, DnsRecord>(
                "SELECT id, zone, name, record_type, value, ttl, created_at
                 FROM dns_records ORDER BY zone, name, record_type",
            )
            .fetch_all(&self.pool)
            .await?)
        }
    }

    pub async fn add_dns_record(
        &self,
        zone: &str,
        name: &str,
        record_type: &str,
        value: &str,
        ttl: i64,
    ) -> anyhow::Result<DnsRecord> {
        let zone = normalize(zone)?;
        let name = normalize_record_name(name)?;
        let record_type = normalize_record_type(record_type)?;
        let value = value.trim().to_string();
        if value.is_empty() {
            anyhow::bail!("value required");
        }
        let ttl = if ttl <= 0 { 3600 } else { ttl };

        sqlx::query(
            "INSERT INTO dns_records (zone, name, record_type, value, ttl, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&zone)
        .bind(&name)
        .bind(&record_type)
        .bind(&value)
        .bind(ttl)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.get_dns_record(&zone, &name, &record_type, &value)
            .await
    }

    pub async fn delete_dns_record(&self, id: i64) -> anyhow::Result<()> {
        let n = sqlx::query("DELETE FROM dns_records WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 {
            anyhow::bail!("record not found");
        }
        Ok(())
    }

    pub fn dns_guide(&self, zone: &str, vps_ip: &str) -> anyhow::Result<DnsGuide> {
        let zone = normalize(zone)?;
        let vps_ip = vps_ip.trim().to_string();
        if vps_ip.is_empty() {
            anyhow::bail!("vps_ip required");
        }
        let ip = vps_ip.clone();

        let nameservers = load_nameservers(&self.root_dir);
        let steps = vec![
            format!("ドメイン取得サイトでゾーン {zone} の DNS 設定を開く"),
            "ネームサーバーを VPS / レジストラの指定に変更（下記参照）".into(),
            format!("A レコード: @ または {zone} → {ip}"),
            format!("サブドメイン例: panel.{zone} → {ip}（A レコード）"),
            "aruaru 管理画面で platform ドメインを登録".into(),
            "aruaru-https で HTTPS 自動設定（別途インストール）".into(),
        ];

        let suggested = vec![
            SuggestedRecord {
                record_type: "A".into(),
                name: "@".into(),
                value: vps_ip.clone(),
                ttl: 3600,
                note: "ルートドメインを VPS に向ける".into(),
            },
            SuggestedRecord {
                record_type: "A".into(),
                name: "www".into(),
                value: vps_ip.clone(),
                ttl: 3600,
                note: "www サブドメイン".into(),
            },
            SuggestedRecord {
                record_type: "A".into(),
                name: "panel".into(),
                value: vps_ip.clone(),
                ttl: 3600,
                note: "管理パネル用サブドメイン例".into(),
            },
        ];

        Ok(DnsGuide {
            zone,
            vps_ip: ip,
            nameservers,
            steps,
            suggested_records: suggested,
        })
    }

    async fn get_dns_record(
        &self,
        zone: &str,
        name: &str,
        record_type: &str,
        value: &str,
    ) -> anyhow::Result<DnsRecord> {
        sqlx::query_as::<_, DnsRecord>(
            "SELECT id, zone, name, record_type, value, ttl, created_at
             FROM dns_records WHERE zone = $1 AND name = $2 AND record_type = $3 AND value = $4",
        )
        .bind(zone)
        .bind(name)
        .bind(record_type)
        .bind(value)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}

fn normalize_record_name(name: &str) -> anyhow::Result<String> {
    let s = name.trim().to_lowercase();
    if s.is_empty() || s == "@" {
        return Ok("@".into());
    }
    if s.contains(' ') {
        anyhow::bail!("invalid record name");
    }
    Ok(s.trim_end_matches('.').to_string())
}

fn normalize_record_type(t: &str) -> anyhow::Result<String> {
    let u = t.trim().to_uppercase();
    if RECORD_TYPES.contains(&u.as_str()) {
        Ok(u)
    } else {
        anyhow::bail!("unsupported record type");
    }
}

fn load_nameservers(root_dir: &std::path::Path) -> Vec<String> {
    let path = root_dir.join("config").join("dns.toml");
    if let Ok(raw) = std::fs::read_to_string(&path) {
        if let Ok(v) = toml::from_str::<DnsConfigFile>(&raw) {
            if !v.nameservers.is_empty() {
                return v.nameservers;
            }
        }
    }
    vec![
        "ns1.example.com".into(),
        "ns2.example.com".into(),
    ]
}

#[derive(serde::Deserialize)]
struct DnsConfigFile {
    #[serde(default)]
    nameservers: Vec<String>,
}
