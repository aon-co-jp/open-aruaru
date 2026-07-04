//! Value Domain DNS API v2
//! https://api.value-domain.com/v2/

use anyhow::Result;
use serde_json::Value;
use super::{DnsRecord, DnsZone, ProviderCredential};

const BASE: &str = "https://api.value-domain.com/v2";

fn build(cred: &ProviderCredential, method: &str, path: &str) -> reqwest::RequestBuilder {
    let url = format!("{}{}", BASE, path);
    let c = reqwest::Client::new();
    let req = match method {
        "GET"    => c.get(&url),
        "POST"   => c.post(&url),
        "PUT"    => c.put(&url),
        "DELETE" => c.delete(&url),
        _        => c.get(&url),
    };
    // Value Domain v2 は Bearer トークン認証
    req.bearer_auth(&cred.api_token)
       .header("Content-Type", "application/json")
}

pub async fn list_zones(cred: &ProviderCredential) -> Result<Vec<DnsZone>> {
    let res: Value = build(cred, "GET", "/domains?limit=100")
        .send().await?.json().await?;
    let zones = res["results"].as_array()
        .ok_or_else(|| anyhow::anyhow!("ドメイン取得失敗: {:?}", res))?
        .iter().map(|d| DnsZone {
            id:     d["domainname"].as_str().unwrap_or("").to_string(),
            name:   d["domainname"].as_str().unwrap_or("").to_string(),
            status: "active".to_string(),
        }).collect();
    Ok(zones)
}

pub async fn list_records(cred: &ProviderCredential, zone_id: &str) -> Result<Vec<DnsRecord>> {
    let res: Value = build(cred, "GET", &format!("/domains/{}/dns", zone_id))
        .send().await?.json().await?;

    // Value Domain は records をテキスト形式で返す場合がある
    let records_text = res["results"]["records"].as_str().unwrap_or("");
    let mut records = vec![];

    for line in records_text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 { continue; }
        let record_type = parts[0].to_uppercase();
        let name = parts[1].to_string();
        let value = parts[2..].join(" ");
        records.push(DnsRecord {
            id:          Some(format!("{}-{}", name, record_type)),
            name,
            record_type,
            value,
            ttl: 3600,
            priority: None,
        });
    }
    Ok(records)
}

pub async fn add_record(cred: &ProviderCredential, zone_id: &str, rec: &DnsRecord) -> Result<String> {
    // 既存レコードを取得して追記する形式
    let existing = list_records(cred, zone_id).await.unwrap_or_default();
    let mut lines: Vec<String> = existing.iter().map(|r| {
        format!("{} {} {}", r.record_type.to_lowercase(), r.name, r.value)
    }).collect();

    let new_line = format!("{} {} {}", rec.record_type.to_lowercase(), rec.name, rec.value);
    if !lines.contains(&new_line) {
        lines.push(new_line);
    }

    let body = serde_json::json!({ "records": lines.join("\n") });
    let res: Value = build(cred, "PUT", &format!("/domains/{}/dns", zone_id))
        .json(&body).send().await?.json().await?;

    if res["status"].as_str() == Some("OK") || res["result"].as_str() == Some("success") {
        Ok(format!("{}-{}", rec.name, rec.record_type))
    } else {
        anyhow::bail!("レコード追加失敗: {:?}", res)
    }
}

pub async fn delete_record(cred: &ProviderCredential, zone_id: &str, record_id: &str) -> Result<()> {
    let existing = list_records(cred, zone_id).await?;
    let lines: Vec<String> = existing.iter()
        .filter(|r| r.id.as_deref() != Some(record_id))
        .map(|r| format!("{} {} {}", r.record_type.to_lowercase(), r.name, r.value))
        .collect();

    let body = serde_json::json!({ "records": lines.join("\n") });
    build(cred, "PUT", &format!("/domains/{}/dns", zone_id))
        .json(&body).send().await?;
    Ok(())
}
