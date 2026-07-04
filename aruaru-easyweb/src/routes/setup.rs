//! 初回セットアップウィザード（ブラウザから設定登録、VI不要）

use poem::{
    handler, get, post,
    web::{Data, Form, Html, Json},
    EndpointExt, Route, Response, Result,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::{db::Pool, settings};

#[derive(Serialize)]
struct IpResult {
    ip:    Option<String>,
    error: Option<String>,
}

/// GET /api/detect-ip — VPS のパブリック IP を自動取得
#[handler]
pub async fn detect_ip() -> Json<IpResult> {
    // 複数のサービスで試みる
    let services = [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
        "https://checkip.amazonaws.com",
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .unwrap_or_default();

    for url in &services {
        if let Ok(res) = client.get(*url).send().await {
            if let Ok(text) = res.text().await {
                let ip = text.trim().to_string();
                if !ip.is_empty() && ip.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    return Json(IpResult { ip: Some(ip), error: None });
                }
            }
        }
    }

    // フォールバック: hostname -I で取得
    if let Ok(out) = tokio::process::Command::new("hostname").arg("-I").output().await {
        let ip = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .map(|s| s.to_string());
        if let Some(ip) = ip {
            return Json(IpResult { ip: Some(ip), error: None });
        }
    }

    Json(IpResult { ip: None, error: Some("IPアドレスを自動取得できませんでした".to_string()) })
}

#[derive(Deserialize)]
struct SetupForm {
    app_url:              String,
    admin_email:          String,
    token_expiry_minutes: String,
    smtp_host:            String,
    smtp_port:            String,
    smtp_username:        String,
    smtp_password:        String,
    smtp_from:            String,
}

/// GET /setup — セットアップ画面表示
#[handler]
async fn setup_page(Data(pool): Data<&Pool>) -> Result<Response> {
    // 既にセットアップ済みならダッシュボードへ
    if settings::is_setup_completed(pool).await {
        return Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/")
            .finish());
    }
    Ok(Response::builder()
        .status(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/setup.html")))
}

/// POST /setup — 設定を DB に保存
#[handler]
async fn setup_save(
    Data(pool): Data<&Pool>,
    Form(f):    Form<SetupForm>,
) -> Result<Response> {
    // 既にセットアップ済みなら拒否
    if settings::is_setup_completed(pool).await {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("セットアップは既に完了しています"));
    }

    let s = settings::AppSettings {
        app_url:              f.app_url.trim().to_string(),
        admin_email:          f.admin_email.trim().to_string(),
        token_expiry_minutes: f.token_expiry_minutes.trim().to_string(),
        smtp_host:            f.smtp_host.trim().to_string(),
        smtp_port:            f.smtp_port.trim().to_string(),
        smtp_username:        f.smtp_username.trim().to_string(),
        smtp_password:        f.smtp_password.trim().to_string(),
        smtp_from:            f.smtp_from.trim().to_string(),
        setup_completed:      true,
    };

    settings::save(pool, &s)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // セットアップ完了 → ログイン画面へ
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/login")
        .finish())
}

/// GET /settings — 設定変更画面（ログイン済みユーザー向け）
#[handler]
async fn settings_page(Data(pool): Data<&Pool>) -> Result<Html<String>> {
    let s = settings::load(pool)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // 設定値を HTML に埋め込む（パスワードは空表示）
    let html = include_str!("../templates/settings.html")
        .replace("{{app_url}}",              &s.app_url)
        .replace("{{admin_email}}",          &s.admin_email)
        .replace("{{token_expiry_minutes}}", &s.token_expiry_minutes)
        .replace("{{smtp_host}}",            &s.smtp_host)
        .replace("{{smtp_port}}",            &s.smtp_port)
        .replace("{{smtp_username}}",        &s.smtp_username)
        .replace("{{smtp_from}}",            &s.smtp_from);

    Ok(Html(html))
}

/// POST /settings — 設定変更保存
#[handler]
async fn settings_save(
    Data(pool): Data<&Pool>,
    Form(f):    Form<SetupForm>,
) -> Result<Response> {
    // パスワードが空欄なら既存を維持
    let current = settings::load(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let password = if f.smtp_password.trim().is_empty() {
        current.smtp_password
    } else {
        f.smtp_password.trim().to_string()
    };

    let s = settings::AppSettings {
        app_url:              f.app_url.trim().to_string(),
        admin_email:          f.admin_email.trim().to_string(),
        token_expiry_minutes: f.token_expiry_minutes.trim().to_string(),
        smtp_host:            f.smtp_host.trim().to_string(),
        smtp_port:            f.smtp_port.trim().to_string(),
        smtp_username:        f.smtp_username.trim().to_string(),
        smtp_password:        password,
        smtp_from:            f.smtp_from.trim().to_string(),
        setup_completed:      true,
    };

    settings::save(pool, &s)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/settings?saved=1")
        .finish())
}

pub fn setup_routes(pool: Pool) -> impl poem::Endpoint {
    Route::new()
        .at("/setup",    get(setup_page).post(setup_save))
        .at("/settings", get(settings_page).post(settings_save))
        .data(pool)
}
