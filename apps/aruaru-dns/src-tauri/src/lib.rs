mod config;
mod db;

use std::sync::Mutex;

use config::AppConfig;
use db::{build_guide, delete_record, insert_record, list_records, sync_all, DualDb};
use tauri::State;

struct AppState {
    db: Mutex<DualDb>,
}

#[derive(serde::Serialize)]
struct DbStatus {
    postgres: bool,
    aruaru_db: bool,
    aruaru_db_path: String,
    postgres_url: String,
    aruaru_db_url: String,
}

#[tauri::command]
fn db_status(state: State<'_, AppState>) -> DbStatus {
    let db = state.db.lock().expect("db lock");
    DbStatus {
        postgres: db.postgres_ok(),
        aruaru_db: db.aruaru_db_ok(),
        aruaru_db_path: db.config.aruaru_db.source_path.clone(),
        postgres_url: db.config.postgres.url.clone(),
        aruaru_db_url: db.config.aruaru_db.url.clone(),
    }
}

#[tauri::command]
async fn list_dns_records(
    state: State<'_, AppState>,
    zone: Option<String>,
) -> Result<Vec<db::DnsRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_records(&db.postgres, zone.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_dns_record(
    state: State<'_, AppState>,
    zone: String,
    name: String,
    record_type: String,
    value: String,
    ttl: i64,
) -> Result<db::DnsRecord, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let rec = insert_record(&db.postgres, &zone, &name, &record_type, &value, ttl)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(ref adb) = db.aruaru_db {
        if db.config.aruaru_db.sync_enabled {
            let _ = insert_record(adb, &zone, &name, &record_type, &value, ttl).await;
        }
    }
    Ok(rec)
}

#[tauri::command]
async fn delete_dns_record(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    delete_record(&db.postgres, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn dns_guide(
    state: State<'_, AppState>,
    zone: String,
    vps_ip: String,
) -> Result<db::DnsGuide, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    build_guide(&zone, &vps_ip, &db.config.nameservers.list).map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_to_aruaru_db(state: State<'_, AppState>) -> Result<u64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let Some(ref adb) = db.aruaru_db else {
        return Err("aruaru-db not connected".into());
    };
    sync_all(&db.postgres, adb)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load().expect("config load");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let dual = rt.block_on(DualDb::connect(config)).expect("db connect");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(dual),
        })
        .invoke_handler(tauri::generate_handler![
            db_status,
            list_dns_records,
            add_dns_record,
            delete_dns_record,
            dns_guide,
            sync_to_aruaru_db,
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
