//! Cloudflare DNS API v4

use anyhow::Result;
use serde_json::Value;
use super::{DnsRecord, DnsZone, ProviderCredential};

const BASE: &str = "https://api.cloudflare.com/client/v4";

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
    req.bearer_auth(&cred.api_token)
       .header("Content-Type", "application/json")
}

pub async fn list_zones(cred: &ProviderCredential) -> Result<Vec<DnsZone>> {
    let res: Value = build(cred, "GET", "/zones")
        .send().await?.json().await?;
    let zones = res["result"].as_array()
        .ok_or_else(|| anyhow::anyhow!("ゾーン取得失敗: {}", res["errors"]))?
        .iter().map(|z| DnsZone {
            id:     z["id"].as_str().unwrap_or("").to_string(),
            name:   z["name"].as_str().unwrap_or("").to_string(),
            status: z["status"].as_str().unwrap_or("").to_string(),
        }).collect();
    Ok(zones)
}

pub async fn list_records(cred: &ProviderCredential, zone_id: &str) -> Result<Vec<DnsRecord>> {
    let res: Value = build(cred, "GET", &format!("/zones/{}/dns_records?per_page=100", zone_id))
        .send().await?.json().await?;
    let records = res["result"].as_array()
        .ok_or_else(|| anyhow::anyhow!("レコード取得失敗"))?
        .iter().map(|r| DnsRecord {
            id:          Some(r["id"].as_str().unwrap_or("").to_string()),
            name:        r["name"].as_str().unwrap_or("").to_string(),
            record_type: r["type"].as_str().unwrap_or("").to_string(),
            value:       r["content"].as_str().unwrap_or("").to_string(),
            ttl:         r["ttl"].as_u64().unwrap_or(3600) as u32,
            priority:    r["priority"].as_u64().map(|p| p as u32),
        }).collect();
    Ok(records)
}

pub async fn add_record(cred: &ProviderCredential, zone_id: &str, rec: &DnsRecord) -> Result<String> {
    let mut body = serde_json::json!({
        "type":    rec.record_type,
        "name":    rec.name,
        "content": rec.value,
        "ttl":     rec.ttl,
        "proxied": false,
    });
    if let Some(p) = rec.priority {
        body["priority"] = serde_json::json!(p);
    }
    let res: Value = build(cred, "POST", &format!("/zones/{}/dns_records", zone_id))
        .json(&body).send().await?.json().await?;

    if res["success"].as_bool().unwrap_or(false) {
        Ok(res["result"]["id"].as_str().unwrap_or("").to_string())
    } else {
        anyhow::bail!("{}", res["errors"][0]["message"].as_str().unwrap_or("不明なエラー"))
    }
}

pub async fn delete_record(cred: &ProviderCredential, zone_id: &str, record_id: &str) -> Result<()> {
    let res: Value = build(cred, "DELETE", &format!("/zones/{}/dns_records/{}", zone_id, record_id))
        .send().await?.json().await?;
    if res["success"].as_bool().unwrap_or(false) { Ok(()) }
    else { anyhow::bail!("{}", res["errors"][0]["message"].as_str().unwrap_or("削除失敗")) }
}
