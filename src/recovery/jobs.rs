use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Backup,
    Restore,
    Sync,
    Migrate,
    BindDrive,
    RestoreAccess,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl JobKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Restore => "restore",
            Self::Sync => "sync",
            Self::Migrate => "migrate",
            Self::BindDrive => "bind_drive",
            Self::RestoreAccess => "restore_access",
        }
    }
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecoveryJob {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
    pub message: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}
