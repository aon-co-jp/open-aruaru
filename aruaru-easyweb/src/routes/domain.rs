use poem::{
    handler, get, post, delete,
    web::{Data, Json, Path},
    EndpointExt, Route, Response, Result,
    http::StatusCode,
};
use serde::Deserialize;
use crate::{config::AppConfig, db::Pool, domain, webserver};

#[derive(Deserialize)]
struct AddReq {
    name:         String,
    is_app_domain: Option<bool>,
    webserver:    Option<String>,
    proxy_port:   Option<u16>,
    https:        Option<bool>,
}

#[derive(Deserialize)]
struct ToggleHttpsReq { enabled: bool }

#[derive(Deserialize)]
struct SwitchWsReq { webserver: String }

#[handler]
async fn list(Data(pool): Data<&Pool>) -> Result<Json<Vec<domain::Domain>>> {
    Ok(Json(domain::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?))
}

/// ドメイン追加 → Web サーバー設定自動生成・適用
#[handler]
async fn add(
    Data(pool): Data<&Pool>,
    Data(cfg):  Data<&AppConfig>,
    Json(req):  Json<AddReq>,
) -> Result<Json<domain::Domain>> {
    let ws      = req.webserver.as_deref().unwrap_or("nginx");
    let https   = req.https.unwrap_or(false);
    let d = domain::add(pool, &req.name, req.is_app_domain.unwrap_or(false), ws, req.proxy_port)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Nginx/Apache 設定を自動生成・書き込み・有効化・リロード（VI不要）
    webserver::apply_domain_config(
        pool, &d.id, &d.name, ws, https,
        req.proxy_port, &cfg.admin_email,
    ).await.map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(d))
}

/// HTTPS トグル → certbot 自動取得 or 設定更新
#[handler]
async fn toggle_https(
    Data(pool): Data<&Pool>,
    Data(cfg):  Data<&AppConfig>,
    Path(id):   Path<String>,
    Json(req):  Json<ToggleHttpsReq>,
) -> Result<Response> {
    let d = domain::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter().find(|x| x.id == id)
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    if req.enabled {
        domain::ssl::setup_https(pool, &id, &d.name, &cfg.admin_email, &d.webserver)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    } else {
        domain::set_https(pool, &id, false).await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    }

    // Nginx/Apache 設定を再生成・リロード
    webserver::apply_domain_config(
        pool, &id, &d.name, &d.webserver, req.enabled,
        d.proxy_port.map(|p| p as u16), &cfg.admin_email,
    ).await.map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder().status(StatusCode::OK).finish())
}

/// Web サーバー切り替え（Apache ↔ Nginx）
#[handler]
async fn switch_ws(
    Data(pool): Data<&Pool>,
    Data(cfg):  Data<&AppConfig>,
    Path(id):   Path<String>,
    Json(req):  Json<SwitchWsReq>,
) -> Result<Response> {
    let d = domain::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
        .into_iter().find(|x| x.id == id)
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    webserver::switch_webserver(
        pool, &id, &d.name, &d.webserver, &req.webserver,
        d.https_enabled, d.proxy_port.map(|p| p as u16), &cfg.admin_email,
    ).await.map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    domain::set_webserver(pool, &id, &req.webserver).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder().status(StatusCode::OK).finish())
}

/// ドメイン削除 → 設定ファイルも自動削除
#[handler]
async fn remove(
    Data(pool): Data<&Pool>,
    Path(id):   Path<String>,
) -> Result<Response> {
    if let Some(d) = domain::delete(pool, &id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
    {
        webserver::remove_domain_config(&d.name, &d.webserver).await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    }
    Ok(Response::builder().status(StatusCode::OK).finish())
}

pub fn domain_routes(pool: Pool, cfg: AppConfig) -> impl poem::Endpoint {
    Route::new()
        .at("/",              get(list).post(add))
        .at("/:id/https",     post(toggle_https))
        .at("/:id/webserver", post(switch_ws))
        .at("/:id",           delete(remove))
        .data(pool)
        .data(cfg)
}
