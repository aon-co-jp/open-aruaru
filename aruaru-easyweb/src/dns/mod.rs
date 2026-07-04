//! DNS プロバイダー統合モジュール
//! ConoHa / Cloudflare / Value Domain 対応

pub mod cloudflare;
pub mod conoha;
pub mod value_domain;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// DNS レコード（共通型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id:       Option<String>, // プロバイダー側の ID
    pub name:     String,         // @ / www / mail など
    pub record_type: String,      // A / AAAA / CNAME / MX / TXT
    pub value:    String,         // IP アドレス / ホスト名など
    pub ttl:      u32,
    pub priority: Option<u32>,    // MX レコード用
}

/// DNS ゾーン情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsZone {
    pub id:     String,
    pub name:   String, // example.com
    pub status: String,
}

/// 対応プロバイダー
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Cloudflare,
    ConoHa,
    ValueDomain,
}

impl Provider {
    pub fn label(&self) -> &str {
        match self {
            Provider::Cloudflare   => "Cloudflare",
            Provider::ConoHa       => "ConoHa",
            Provider::ValueDomain  => "Value Domain",
        }
    }
}

/// プロバイダー認証情報（DB 保存用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredential {
    pub provider:  String,
    pub api_token: String,  // Cloudflare: API Token / ConoHa: API Token
    pub api_key:   String,  // Value Domain: API Key / ConoHa: テナント ID
    pub email:     String,  // Cloudflare のみ使用
}

/// ゾーン一覧取得
pub async fn list_zones(cred: &ProviderCredential) -> Result<Vec<DnsZone>> {
    match cred.provider.as_str() {
        "cloudflare"   => cloudflare::list_zones(cred).await,
        "conoha"       => conoha::list_zones(cred).await,
        "value_domain" => value_domain::list_zones(cred).await,
        p => anyhow::bail!("未対応プロバイダー: {}", p),
    }
}

/// DNS レコード一覧
pub async fn list_records(cred: &ProviderCredential, zone_id: &str) -> Result<Vec<DnsRecord>> {
    match cred.provider.as_str() {
        "cloudflare"   => cloudflare::list_records(cred, zone_id).await,
        "conoha"       => conoha::list_records(cred, zone_id).await,
        "value_domain" => value_domain::list_records(cred, zone_id).await,
        p => anyhow::bail!("未対応プロバイダー: {}", p),
    }
}

/// DNS レコード追加
pub async fn add_record(cred: &ProviderCredential, zone_id: &str, rec: &DnsRecord) -> Result<String> {
    match cred.provider.as_str() {
        "cloudflare"   => cloudflare::add_record(cred, zone_id, rec).await,
        "conoha"       => conoha::add_record(cred, zone_id, rec).await,
        "value_domain" => value_domain::add_record(cred, zone_id, rec).await,
        p => anyhow::bail!("未対応プロバイダー: {}", p),
    }
}

/// DNS レコード削除
pub async fn delete_record(cred: &ProviderCredential, zone_id: &str, record_id: &str) -> Result<()> {
    match cred.provider.as_str() {
        "cloudflare"   => cloudflare::delete_record(cred, zone_id, record_id).await,
        "conoha"       => conoha::delete_record(cred, zone_id, record_id).await,
        "value_domain" => value_domain::delete_record(cred, zone_id, record_id).await,
        p => anyhow::bail!("未対応プロバイダー: {}", p),
    }
}

/// VPS の IP → A レコードを自動登録
pub async fn auto_register_a(
    cred: &ProviderCredential,
    zone_id: &str,
    domain: &str,
    vps_ip: &str,
) -> Result<Vec<String>> {
    let mut ids = vec![];
    for name in &[domain, &format!("www.{}", domain)] {
        let rec = DnsRecord {
            id: None,
            name: name.to_string(),
            record_type: "A".to_string(),
            value: vps_ip.to_string(),
            ttl: 3600,
            priority: None,
        };
        match add_record(cred, zone_id, &rec).await {
            Ok(id) => ids.push(id),
            Err(e) => tracing::warn!("A レコード追加失敗 {}: {}", name, e),
        }
    }
    Ok(ids)
}
