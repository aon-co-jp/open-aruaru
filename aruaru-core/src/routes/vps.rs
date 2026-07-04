use poem::{
    handler, get, post, delete,
    web::{Data, Json, Path},
    Route, Response, Result, http::StatusCode,
};
use serde::Deserialize;
use crate::{db::Pool, vps};

#[derive(Deserialize)]
struct RegisterReq {
    name:        String,
    ip_address:  String,          // Webフォームから入力するだけ
    port:        Option<u16>,
    username:    Option<String>,
    auth_method: Option<String>,
    key_id:      Option<String>,
}

#[handler]
async fn list(Data(pool): Data<&Pool>) -> Result<Json<Vec<vps::VpsServer>>> {
    Ok(Json(vps::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?))
}

/// VPS 登録（IP アドレスを入力するだけ）
#[handler]
async fn register(Data(pool): Data<&Pool>, Json(req): Json<RegisterReq>) -> Result<Json<vps::VpsServer>> {
    let server = vps::register(
        pool,
        &req.name,
        &req.ip_address,
        req.port.unwrap_or(22),
        req.username.as_deref().unwrap_or("root"),
        req.auth_method.as_deref().unwrap_or("key"),
        req.key_id.as_deref(),
    ).await.map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(server))
}

/// 接続確認
#[handler]
async fn ping(Data(pool): Data<&Pool>, Path(id): Path<String>) -> Result<Json<serde_json::Value>> {
    let server = vps::get(pool, &id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    let online = vps::check_connectivity(&server.ip_address, server.port as u16).await;
    let status = if online { "online" } else { "offline" };
    vps::update_status(pool, &id, status).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(serde_json::json!({ "status": status })))
}

#[handler]
async fn remove(Data(pool): Data<&Pool>, Path(id): Path<String>) -> Result<Response> {
    vps::delete(pool, &id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Response::builder().status(StatusCode::OK).finish())
}

pub fn vps_routes(pool: Pool) -> Route {
    Route::new()
        .at("/",         get(list).data(pool.clone()).post(register).data(pool.clone()))
        .at("/:id/ping", post(ping).data(pool.clone()))
        .at("/:id",      delete(remove).data(pool.clone()))
        .at("/detect-ip",  get(detect_ip))
}

/// GET /api/vps/detect-ip — VPS の外部 IP を自動検出
#[handler]
async fn detect_ip() -> Result<Json<serde_json::Value>> {
    // 複数の IP 検出サービスを試みる（どれかが成功すれば返す）
    let services = [
        "https://api.ipify.org?format=json",
        "https://api4.my-ip.io/v2/ip.json",
        "https://ipinfo.io/json",
    ];
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    for url in &services {
        if let Ok(res) = client.get(*url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                let ip = data["ip"].as_str()
                    .or(data["ipv4"].as_str())
                    .unwrap_or("")
                    .to_string();
                if !ip.is_empty() {
                    return Ok(Json(serde_json::json!({ "ip": ip })));
                }
            }
        }
    }
    Ok(Json(serde_json::json!({ "ip": "" })))
}
