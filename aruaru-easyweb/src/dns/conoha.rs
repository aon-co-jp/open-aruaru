//! ConoHa DNS API v1
//! ドキュメント: https://www.conoha.jp/docs/dns-api/

use anyhow::Result;
use serde_json::Value;
use super::{DnsRecord, DnsZone, ProviderCredential};

// ConoHa DNS エンドポイント（東京リージョン）
const BASE: &str = "https://dns-service.tyo1.conoha.io/v1";

/// ConoHa は Identity API でトークンを取得する
async fn get_token(cred: &ProviderCredential) -> Result<String> {
    // api_key = ユーザー名, api_token = パスワード, email = テナント名
    let body = serde_json::json!({
        "auth": {
            "passwordCredentials": {
                "username": cred.api_key,
                "password": cred.api_token,
            },
            "tenantName": cred.email,
        }
    });
    let res: Value = reqwest::Client::new()
        .post("https://identity.tyo1.conoha.io/v2.0/tokens")
        .json(&body).send().await?.json().await?;

    res["access"]["token"]["id"].as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("ConoHa 認証失敗: {}", res))
}

fn build(token: &str, method: &str, path: &str) -> reqwest::RequestBuilder {
    let url = format!("{}{}", BASE, path);
    let c = reqwest::Client::new();
    let req = match method {
        "GET"    => c.get(&url),
        "POST"   => c.post(&url),
        "PUT"    => c.put(&url),
        "DELETE" => c.delete(&url),
        _        => c.get(&url),
    };
    req.header("X-Auth-Token", token)
       .header("Content-Type", "application/json")
}

pub async fn list_zones(cred: &ProviderCredential) -> Result<Vec<DnsZone>> {
    let token = get_token(cred).await?;
    let res: Value = build(&token, "GET", "/zones").send().await?.json().await?;
    let zones = res["zones"].as_array()
        .ok_or_else(|| anyhow::anyhow!("ゾーン取得失敗"))?
        .iter().map(|z| DnsZone {
            id:     z["id"].as_str().unwrap_or("").to_string(),
            name:   z["name"].as_str().unwrap_or("").trim_end_matches('.').to_string(),
            status: z["status"].as_str().unwrap_or("").to_string(),
        }).collect();
    Ok(zones)
}

pub async fn list_records(cred: &ProviderCredential, zone_id: &str) -> Result<Vec<DnsRecord>> {
    let token = get_token(cred).await?;
    let res: Value = build(&token, "GET", &format!("/zones/{}/recordsets", zone_id))
        .send().await?.json().await?;
    let records = res["recordsets"].as_array()
        .ok_or_else(|| anyhow::anyhow!("レコード取得失敗"))?
        .iter().flat_map(|r| {
            let name        = r["name"].as_str().unwrap_or("").trim_end_matches('.').to_string();
            let record_type = r["type"].as_str().unwrap_or("").to_string();
            let id          = r["id"].as_str().unwrap_or("").to_string();
            let ttl         = r["ttl"].as_u64().unwrap_or(3600) as u32;
            r["records"].as_array().unwrap_or(&vec![]).iter().map(move |v| DnsRecord {
                id:          Some(id.clone()),
                name:        name.clone(),
                record_type: record_type.clone(),
                value:       v.as_str().unwrap_or("").to_string(),
                ttl,
                priority:    None,
            }).collect::<Vec<_>>()
        }).collect();
    Ok(records)
}

pub async fn add_record(cred: &ProviderCredential, zone_id: &str, rec: &DnsRecord) -> Result<String> {
    let token = get_token(cred).await?;
    let name = if rec.name.ends_with('.') { rec.name.clone() }
               else { format!("{}.", rec.name) };
    let body = serde_json::json!({
        "name":    name,
        "type":    rec.record_type,
        "ttl":     rec.ttl,
        "records": [rec.value],
    });
    let res: Value = build(&token, "POST", &format!("/zones/{}/recordsets", zone_id))
        .json(&body).send().await?.json().await?;
    Ok(res["id"].as_str().unwrap_or("").to_string())
}

pub async fn delete_record(cred: &ProviderCredential, zone_id: &str, record_id: &str) -> Result<()> {
    let token = get_token(cred).await?;
    let status = build(&token, "DELETE", &format!("/zones/{}/recordsets/{}", zone_id, record_id))
        .send().await?.status();
    if status.is_success() { Ok(()) }
    else { anyhow::bail!("削除失敗: HTTP {}", status) }
}
