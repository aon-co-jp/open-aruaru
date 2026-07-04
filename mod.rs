//! aruaru-easyweb にマウントされる Anthropic 接続ルート

use poem::{
    handler, get, post,
    web::{Data, Form, Html},
    Route, Response, Result, http::StatusCode,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::agent;

#[derive(Deserialize)]
struct ConnectForm {
    api_key: String,
    // plan / billing は hidden フィールドで送られるが省略可能
    #[serde(default)]
    plan:    String,
    #[serde(default)]
    billing: String,
    #[serde(default)]
    account: String,
}

/// GET /anthropic — Anthropic ログイン画面
#[handler]
async fn anthropic_page(Data(pool): Data<&SqlitePool>) -> Result<Html<String>> {
    let info = agent::get_plan_info(pool).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let html = include_str!("../templates/anthropic.html");
    let html = if let Some(p) = info {
        html.replace("{{connected}}", "true")
            .replace("{{plan}}",    &p.plan)
            .replace("{{billing}}", &p.billing)
            .replace("{{account}}", &p.account)
    } else {
        html.replace("{{connected}}", "false")
            .replace("{{plan}}",    "")
            .replace("{{billing}}", "")
            .replace("{{account}}", "")
    };
    Ok(Html(html))
}

/// POST /anthropic/connect — API Key 検証 & 保存
#[handler]
async fn connect(Data(pool): Data<&SqlitePool>, Form(f): Form<ConnectForm>) -> Result<Response> {
    match agent::connect_anthropic(pool, &f.api_key, &f.plan, &f.billing, &f.account).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/anthropic?connected=1")
            .finish()),
        Err(e) => Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", format!("/anthropic?error={}", urlencoding(e.to_string())))
            .finish()),
    }
}

/// POST /anthropic/disconnect — 接続解除
#[handler]
async fn disconnect(Data(pool): Data<&SqlitePool>) -> Result<Response> {
    sqlx::query!(
        "UPDATE ai_credentials SET api_key='',verified=0,connected_at=NULL WHERE id='anthropic'"
    ).execute(pool).await
     .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Response::builder().status(StatusCode::FOUND).header("Location", "/anthropic").finish())
}

pub fn ai_routes(pool: SqlitePool) -> Route {
    Route::new()
        .at("/anthropic",            get(anthropic_page).data(pool.clone())
                                         .post(connect).data(pool.clone()))
        .at("/anthropic/disconnect", post(disconnect).data(pool.clone()))
}

fn urlencoding(s: String) -> String {
    s.replace(' ', "%20").replace(':', "%3A").replace('"', "%22")
}
