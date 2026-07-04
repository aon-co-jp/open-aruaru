//! aruaru-ai-core REST API
//! aruaru-core から呼び出される

use poem::{
    handler, post,
    web::Json,
    Route, Result, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::{agent, prompt};

#[derive(Deserialize)]
struct ExecuteReq {
    vps_id:      String,
    instruction: String,
    os_info:     Option<String>,
}

#[derive(Deserialize)]
struct FileOpReq {
    vps_id:       String,
    instruction:  String,
    current_path: String,
}

#[derive(Serialize)]
struct ExecuteRes {
    commands:    Vec<String>,
    description: String,
    safe:        bool,
}

/// POST /api/ai/execute — 自然言語 → シェルコマンド変換
#[handler]
async fn execute(Json(req): Json<ExecuteReq>) -> Result<Json<ExecuteRes>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| poem::Error::from_string("ANTHROPIC_API_KEY not set", StatusCode::INTERNAL_SERVER_ERROR))?;

    let os_info = req.os_info.as_deref().unwrap_or("Ubuntu 22.04 LTS");
    let p       = prompt::build_command_prompt(&req.instruction, os_info);
    let result  = agent::generate_commands(&p, &api_key).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(ExecuteRes {
        commands:    result.commands,
        description: result.description,
        safe:        result.safe,
    }))
}

/// POST /api/ai/file-operation — 自然言語 → SFTP 操作変換
#[handler]
async fn file_operation(Json(req): Json<FileOpReq>) -> Result<Json<serde_json::Value>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| poem::Error::from_string("ANTHROPIC_API_KEY not set", StatusCode::INTERNAL_SERVER_ERROR))?;

    let p      = prompt::build_sftp_prompt(&req.instruction, &req.current_path);
    let result = agent::generate_commands(&p, &api_key).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(serde_json::json!({ "message": result.description, "safe": result.safe })))
}

pub fn ai_routes() -> Route {
    Route::new()
        .at("/execute",        post(execute))
        .at("/file-operation", post(file_operation))
}
