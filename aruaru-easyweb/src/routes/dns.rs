//! DNS 管理ルート

use poem::{
    handler, get, post, delete,
    web::{Data, Form, Path, Query},
    EndpointExt, Route, Response, Result,
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::{db::Pool, dns::{self, DnsRecord, ProviderCredential}};

// ---- リクエスト/レスポンス型 ----

#[derive(Deserialize)]
pub struct SaveProviderForm {
    pub provider:  String,
    pub label:     String,
    pub api_token: String,
    pub api_key:   String,
    pub email:     String,
}

#[derive(Deserialize)]
pub struct AddRecordForm {
    pub provider_id:  String,
    pub zone_id:      String,
    pub name:         String,
    pub record_type:  String,
    pub value:        String,
    pub ttl:          Option<u32>,
    pub priority:     Option<u32>,
}

#[derive(Deserialize)]
pub struct ZoneQuery { pub provider_id: String }

#[derive(Deserialize)]
pub struct RecordsQuery { pub provider_id: String, pub zone_id: String }

#[derive(Deserialize)]
pub struct AutoRegisterForm {
    pub provider_id: String,
    pub zone_id:     String,
    pub domain:      String,
    pub vps_ip:      String,
}

// ---- DB ヘルパー ----

async fn load_credential(pool: &Pool, provider_id: &str) -> anyhow::Result<ProviderCredential> {
    let row = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT provider, api_token, api_key, email, label FROM dns_providers WHERE id=?"
    ).bind(provider_id).fetch_one(pool).await?;
    Ok(ProviderCredential {
        provider:  row.0,
        api_token: row.1,
        api_key:   row.2,
        email:     row.3,
    })
}

// ---- ハンドラ ----

/// GET /dns — DNS 管理ページ
#[handler]
pub async fn dns_page() -> Result<poem::Response> {
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/dns.html")))
}

/// GET /api/dns/providers — 登録済みプロバイダー一覧
#[handler]
pub async fn list_providers(Data(pool): Data<&Pool>) -> Result<Response> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, provider, label FROM dns_providers ORDER BY created_at"
    ).fetch_all(pool).await
     .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let list: Vec<_> = rows.iter().map(|(id, provider, label)| {
        serde_json::json!({ "id": id, "provider": provider, "label": label })
    }).collect();

    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&list).unwrap_or_default()))
}

/// POST /api/dns/providers — プロバイダー登録
#[handler]
pub async fn save_provider(
    Data(pool): Data<&Pool>,
    Form(f): Form<SaveProviderForm>,
) -> Result<Response> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO dns_providers (id, provider, label, api_token, api_key, email) VALUES (?,?,?,?,?,?)"
    ).bind(&id).bind(&f.provider).bind(&f.label)
     .bind(&f.api_token).bind(&f.api_key).bind(&f.email)
     .execute(pool).await
     .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::json!({ "ok": true, "id": id }).to_string()))
}

/// DELETE /api/dns/providers/:id — プロバイダー削除
#[handler]
pub async fn delete_provider(Data(pool): Data<&Pool>, Path(id): Path<String>) -> Result<Response> {
    sqlx::query("DELETE FROM dns_providers WHERE id=?")
        .bind(&id).execute(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(r#"{"ok":true}"#))
}

/// GET /api/dns/zones?provider_id=xxx — ゾーン一覧
#[handler]
pub async fn list_zones(
    Data(pool): Data<&Pool>,
    Query(q): Query<ZoneQuery>,
) -> Result<Response> {
    let cred = load_credential(pool, &q.provider_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
    let zones = dns::list_zones(&cred).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&zones).unwrap_or_default()))
}

/// GET /api/dns/records?provider_id=xxx&zone_id=yyy — レコード一覧
#[handler]
pub async fn list_records(
    Data(pool): Data<&Pool>,
    Query(q): Query<RecordsQuery>,
) -> Result<Response> {
    let cred = load_credential(pool, &q.provider_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
    let records = dns::list_records(&cred, &q.zone_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&records).unwrap_or_default()))
}

/// POST /api/dns/records — レコード追加
#[handler]
pub async fn add_record(
    Data(pool): Data<&Pool>,
    Form(f): Form<AddRecordForm>,
) -> Result<Response> {
    let cred = load_credential(pool, &f.provider_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
    let rec = DnsRecord {
        id: None,
        name: f.name.clone(),
        record_type: f.record_type.clone(),
        value: f.value.clone(),
        ttl: f.ttl.unwrap_or(3600),
        priority: f.priority,
    };
    let id = dns::add_record(&cred, &f.zone_id, &rec).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::json!({ "ok": true, "id": id }).to_string()))
}

/// DELETE /api/dns/records/:record_id?provider_id=xxx&zone_id=yyy
#[handler]
pub async fn delete_record(
    Data(pool): Data<&Pool>,
    Path(record_id): Path<String>,
    Query(q): Query<RecordsQuery>,
) -> Result<Response> {
    let cred = load_credential(pool, &q.provider_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
    dns::delete_record(&cred, &q.zone_id, &record_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(r#"{"ok":true}"#))
}

/// POST /api/dns/auto-register — VPS IP を A レコードとして自動登録
#[handler]
pub async fn auto_register(
    Data(pool): Data<&Pool>,
    Form(f): Form<AutoRegisterForm>,
) -> Result<Response> {
    let cred = load_credential(pool, &f.provider_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
    let ids = dns::auto_register_a(&cred, &f.zone_id, &f.domain, &f.vps_ip).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
    Ok(Response::builder().status(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::json!({ "ok": true, "created": ids.len(), "ids": ids }).to_string()))
}

pub fn dns_api_routes(pool: Pool) -> impl poem::Endpoint {
    Route::new()
        .at("/providers",          get(list_providers).post(save_provider))
        .at("/providers/:id",      delete(delete_provider))
        .at("/zones",              get(list_zones))
        .at("/records",            get(list_records).post(add_record))
        .at("/records/:record_id", delete(delete_record))
        .at("/auto-register",      post(auto_register))
        .data(pool)
}
