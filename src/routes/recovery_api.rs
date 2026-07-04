use poem::{
    handler, post,
    web::{Data, Json},
    IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

use crate::recovery::RecoveryService;
use crate::state::SharedState;

#[handler]
pub async fn recovery_status(state: Data<&SharedState>) -> impl IntoResponse {
    let installed = state.has_plugin(crate::recovery::PLUGIN_ID).await.unwrap_or(false);
    if !installed {
        return Json(json!({
            "ok": true,
            "installed": false,
            "plugin": crate::recovery::PLUGIN_ID
        }));
    }
    let svc = RecoveryService::new(&state);
    let cfg = svc.load_config().ok();
    Json(json!({
        "ok": true,
        "installed": true,
        "plugin": crate::recovery::PLUGIN_ID,
        "config": cfg
    }))
}

#[handler]
pub async fn recovery_scan(state: Data<&SharedState>) -> impl IntoResponse {
    match RecoveryService::new(&state).scan_footprints() {
        Ok(items) => Json(json!({ "ok": true, "footprints": items })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct BindDriveBody {
    data_dir: String,
    app_dir: String,
}

#[handler]
pub async fn recovery_bind(state: Data<&SharedState>, Json(body): Json<BindDriveBody>) -> impl IntoResponse {
    let svc = RecoveryService::new(&state);
    match svc
        .bind_drive(PathBuf::from(&body.data_dir).as_path(), PathBuf::from(&body.app_dir).as_path())
        .await
    {
        Ok(fp) => Json(json!({ "ok": true, "footprint": fp })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct RestoreAccessBody {
    marker_path: String,
}

#[handler]
pub async fn recovery_restore(state: Data<&SharedState>, Json(body): Json<RestoreAccessBody>) -> impl IntoResponse {
    let svc = RecoveryService::new(&state);
    match svc
        .restore_access(PathBuf::from(&body.marker_path).as_path())
        .await
    {
        Ok(path) => Json(json!({ "ok": true, "desktop_config": path.to_string_lossy() })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn recovery_jobs(state: Data<&SharedState>) -> impl IntoResponse {
    let svc = RecoveryService::new(&state);
    match svc.list_jobs(50).await {
        Ok(jobs) => Json(json!({ "ok": true, "jobs": jobs })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[handler]
pub async fn recovery_footprints_db(state: Data<&SharedState>) -> impl IntoResponse {
    let svc = RecoveryService::new(&state);
    match svc.list_footprints_db().await {
        Ok(rows) => Json(json!({ "ok": true, "footprints": rows })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
pub struct UninstallBody {
    id: String,
    #[serde(default)]
    purge: bool,
}

#[handler]
pub async fn uninstall_api(state: Data<&SharedState>, Json(body): Json<UninstallBody>) -> impl IntoResponse {
    match state.uninstall(&body.id, body.purge).await {
        Ok(()) => Json(json!({ "ok": true, "uninstalled": body.id })),
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

pub fn recovery_routes() -> poem::Route {
    poem::Route::new()
        .at("/status", poem::get(recovery_status))
        .at("/scan", poem::get(recovery_scan))
        .at("/bind", post(recovery_bind))
        .at("/restore", post(recovery_restore))
        .at("/jobs", poem::get(recovery_jobs))
        .at("/footprints", poem::get(recovery_footprints_db))
}
