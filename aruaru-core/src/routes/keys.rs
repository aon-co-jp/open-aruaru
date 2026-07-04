use poem::{
    handler, get, post, delete,
    web::{Data, Json, Path},
    Route, Response, Result, http::StatusCode,
};
use serde::Deserialize;
use crate::{db::Pool, keygen};

#[derive(Deserialize)]
struct GenReq { name: String, key_type: Option<String> }

/// 鍵一覧
#[handler]
async fn list(Data(pool): Data<&Pool>) -> Result<Json<Vec<keygen::KeyPair>>> {
    Ok(Json(keygen::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?))
}

/// 鍵ペアを自動生成（Ed25519 推奨 / RSA 選択可）
#[handler]
async fn generate(Data(pool): Data<&Pool>, Json(req): Json<GenReq>) -> Result<Json<keygen::KeyPair>> {
    let pair = match req.key_type.as_deref().unwrap_or("ed25519") {
        "rsa" => keygen::generate_rsa(pool, &req.name).await,
        _     => keygen::generate_ed25519(pool, &req.name).await,
    }.map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(pair))
}

/// 公開鍵をダウンロード
#[handler]
async fn download_pubkey(
    Data(pool): Data<&Pool>,
    Path(id): Path<String>,
) -> Result<Response> {
    let rows = keygen::list(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let kp = rows.into_iter().find(|k| k.id == id)
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Disposition", format!("attachment; filename=\"{}.pub\"", kp.name))
        .header("Content-Type", "text/plain")
        .body(kp.public_key))
}

#[handler]
async fn remove(Data(pool): Data<&Pool>, Path(id): Path<String>) -> Result<Response> {
    keygen::delete(pool, &id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Response::builder().status(StatusCode::OK).finish())
}

pub fn key_routes(pool: Pool) -> Route {
    Route::new()
        .at("/",              get(list).data(pool.clone()).post(generate).data(pool.clone()))
        .at("/:id/pubkey",    get(download_pubkey).data(pool.clone()))
        .at("/:id",           delete(remove).data(pool.clone()))
}
