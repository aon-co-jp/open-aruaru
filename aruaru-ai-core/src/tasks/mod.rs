//! AI タスクキュー管理

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiTask {
    pub id:        String,
    pub vps_id:    String,
    pub task_type: String,
    pub payload:   String,
    pub status:    String,
    pub result:    Option<String>,
}
