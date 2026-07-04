//! 型・永続化・ハンドラを1ファイルにまとめる（仕様上の分割は `main.rs` のみ）。

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

/// レンタルサーバー登録の上限（件）
pub const MAX_RENTAL_SERVERS: usize = 20_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RentalServer {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub notes: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Persisted {
    servers: Vec<RentalServer>,
    backup_paths: Vec<String>,
}

pub struct AppState {
    data_dir: PathBuf,
    servers: Vec<RentalServer>,
    backup_paths: Vec<PathBuf>,
}

impl AppState {
    pub fn load_or_default(data_dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(data_dir)?;
        let path = data_dir.join("store.json");
        if path.exists() {
            let raw = fs::read_to_string(&path)?;
            let p: Persisted = serde_json::from_str(&raw).unwrap_or_default();
            Ok(Self {
                data_dir: data_dir.to_path_buf(),
                servers: p.servers,
                backup_paths: p.backup_paths.into_iter().map(PathBuf::from).collect(),
            })
        } else {
            Ok(Self {
                data_dir: data_dir.to_path_buf(),
                servers: Vec::new(),
                backup_paths: Vec::new(),
            })
        }
    }

    fn store_path(&self) -> PathBuf {
        self.data_dir.join("store.json")
    }

    fn save(&self) -> std::io::Result<()> {
        let p = Persisted {
            servers: self.servers.clone(),
            backup_paths: self
                .backup_paths
                .iter()
                .map(|x| x.to_string_lossy().to_string())
                .collect(),
        };
        let tmp = self.data_dir.join("store.json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&p)?)?;
        fs::rename(&tmp, self.store_path())?;
        Ok(())
    }
}

type Shared = Arc<RwLock<AppState>>;

fn err(e: impl ToString) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
}

pub async fn root() -> Html<&'static str> {
    Html(
        r#"<!doctype html><meta charset="utf-8"><title>Rental Sync Hub</title>
<h1>Rental Sync Hub</h1>
<p>API: <code>GET /api/health</code>, <code>/api/servers</code>, <code>POST /api/backup</code>, <code>GET /api/git/check</code></p>"#,
    )
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn list_servers(State(st): State<Shared>) -> Result<Json<Vec<RentalServer>>, Response> {
    let g = st.read().await;
    Ok(Json(g.servers.clone()))
}

#[derive(Deserialize)]
pub struct CreateBody {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    #[serde(default)]
    pub notes: String,
}

pub async fn create_server(
    State(st): State<Shared>,
    Json(body): Json<CreateBody>,
) -> Result<Json<RentalServer>, Response> {
    let mut g = st.write().await;
    if g.servers.len() >= MAX_RENTAL_SERVERS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("上限 {} 件に達しています", MAX_RENTAL_SERVERS),
        )
            .into_response());
    }
    let now = now_unix();
    let s = RentalServer {
        id: Uuid::new_v4().to_string(),
        name: body.name,
        host: body.host,
        port: body.port,
        login: body.login,
        notes: body.notes,
        updated_at: now,
    };
    g.servers.push(s.clone());
    g.save().map_err(err)?;
    Ok(Json(s))
}

pub async fn get_server(
    State(st): State<Shared>,
    AxPath(id): AxPath<String>,
) -> Result<Json<RentalServer>, Response> {
    let g = st.read().await;
    let s = g
        .servers
        .iter()
        .find(|x| x.id == id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "not found").into_response())?;
    Ok(Json(s))
}

pub async fn delete_server(
    State(st): State<Shared>,
    AxPath(id): AxPath<String>,
) -> Result<StatusCode, Response> {
    let mut g = st.write().await;
    let n = g.servers.len();
    g.servers.retain(|x| x.id != id);
    if g.servers.len() == n {
        return Err((StatusCode::NOT_FOUND, "not found").into_response());
    }
    g.save().map_err(err)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct BackupPathsBody {
    pub paths: Vec<String>,
}

pub async fn get_backup_paths(State(st): State<Shared>) -> Result<Json<Vec<String>>, Response> {
    let g = st.read().await;
    let v: Vec<String> = g
        .backup_paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    Ok(Json(v))
}

pub async fn set_backup_paths(
    State(st): State<Shared>,
    Json(body): Json<BackupPathsBody>,
) -> Result<StatusCode, Response> {
    let mut g = st.write().await;
    g.backup_paths = body.paths.into_iter().map(PathBuf::from).collect();
    g.save().map_err(err)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `store.json` を各バックアップ先にコピー（NAS / Google Drive 同期フォルダ / USB などのパスを登録）
pub async fn run_backup(State(st): State<Shared>) -> Result<Json<serde_json::Value>, Response> {
    let g = st.read().await;
    let src = g.store_path();
    if !src.exists() {
        return Err((StatusCode::BAD_REQUEST, "store missing").into_response());
    }
    let mut copied = Vec::new();
    let mut errors = Vec::new();
    for dest_root in &g.backup_paths {
        let _ = fs::create_dir_all(dest_root);
        let target = dest_root.join("rental-sync-hub-store.json");
        match fs::copy(&src, &target) {
            Ok(_) => copied.push(target.to_string_lossy().to_string()),
            Err(e) => errors.push(format!("{}: {}", target.display(), e)),
        }
    }
    Ok(Json(serde_json::json!({ "copied": copied, "errors": errors })))
}

#[derive(Serialize)]
pub struct GitCheck {
    pub installed: bool,
    pub version_line: Option<String>,
}

pub async fn git_check() -> Json<GitCheck> {
    let out = Command::new("git").arg("--version").output();
    match out {
        Ok(o) if o.status.success() => Json(GitCheck {
            installed: true,
            version_line: Some(String::from_utf8_lossy(&o.stdout).trim().to_string()),
        }),
        _ => Json(GitCheck {
            installed: false,
            version_line: None,
        }),
    }
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
