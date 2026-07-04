use poem::{
    handler, get, post, delete,
    web::{Data, Json, Multipart, Path, Query},
    Route, Response, Result, http::StatusCode,
};
use serde::Deserialize;
use crate::{db::Pool, keygen, sftp, vps};

#[derive(Deserialize)]
struct PathQuery { path: String }

async fn get_sftp(pool: &Pool, vps_id: &str) -> anyhow::Result<sftp::SftpSession> {
    let server = vps::get(pool, vps_id).await?.ok_or_else(|| anyhow::anyhow!("VPS not found"))?;
    let pkey   = if let Some(kid) = &server.key_id {
        keygen::get_private_key(pool, kid).await?.as_deref().map(String::from)
    } else { None };
    sftp::connect(
        &server.ip_address, server.port as u16, &server.username,
        pkey.as_deref(), None,
    ).await
}

/// ディレクトリ一覧
#[handler]
async fn list_dir(
    Data(pool): Data<&Pool>,
    Path(vps_id): Path<String>,
    Query(q): Query<PathQuery>,
) -> Result<Json<Vec<sftp::FileEntry>>> {
    let session = get_sftp(pool, &vps_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(sftp::list_dir(&session, &q.path).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?))
}

/// ファイルダウンロード
#[handler]
async fn download(
    Data(pool): Data<&Pool>,
    Path(vps_id): Path<String>,
    Query(q): Query<PathQuery>,
) -> Result<Response> {
    let session = get_sftp(pool, &vps_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let data = sftp::download(&session, &q.path).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let filename = q.path.split('/').last().unwrap_or("file");
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .header("Content-Type", "application/octet-stream")
        .body(data))
}

/// ファイルアップロード（マルチパート）
#[handler]
async fn upload(
    Data(pool): Data<&Pool>,
    Path(vps_id): Path<String>,
    Query(q): Query<PathQuery>,
    mut multipart: Multipart,
) -> Result<Response> {
    let session = get_sftp(pool, &vps_id).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = field.file_name().unwrap_or("upload").to_string();
        let data = field.bytes().await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        let remote = format!("{}/{}", q.path.trim_end_matches('/'), filename);
        sftp::upload(&session, &remote, &data).await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    }
    Ok(Response::builder().status(StatusCode::OK).finish())
}

pub fn sftp_routes(pool: Pool) -> Route {
    Route::new()
        .at("/:vps_id/ls",       get(list_dir).data(pool.clone()))
        .at("/:vps_id/download", get(download).data(pool.clone()))
        .at("/:vps_id/upload",   post(upload).data(pool.clone()))
}
