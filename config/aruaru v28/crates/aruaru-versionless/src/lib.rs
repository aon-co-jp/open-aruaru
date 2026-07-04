use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionlessFunction {
    pub id: Uuid,
    pub name: String,
    pub capability: String,
    pub current_revision: String,
    pub enabled: bool,
    pub no_rest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInvocation {
    pub function_name: String,
    pub input: Value,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionOutput {
    pub function_name: String,
    pub ok: bool,
    pub output: Value,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionRegistry { functions: Vec<VersionlessFunction> }

impl FunctionRegistry {
    pub fn with_builtin() -> Self {
        let names = [
            ("domain-register", "domain.write"),
            ("server-register", "server.write"),
            ("device-link", "device.write"),
            ("drive-indexer", "drive.read.allowlisted"),
            ("github-crawler", "github.read.git"),
            ("offline-prefetch", "cache.write.pwa"),
            ("aruaru-project-bundler", "project.bundle"),
            ("readme-report-writer", "report.write"),
            ("report-publisher", "report.publish.graphql"),
            ("ai-router", "ai.route"),
        ];
        Self { functions: names.into_iter().map(|(name, capability)| VersionlessFunction { id: Uuid::new_v4(), name: name.to_string(), capability: capability.to_string(), current_revision: "vless-2026-06-v28".to_string(), enabled: true, no_rest: true }).collect() }
    }
    pub fn list(&self) -> &[VersionlessFunction] { &self.functions }
    pub fn find(&self, name: &str) -> Option<&VersionlessFunction> { self.functions.iter().find(|f| f.name == name && f.enabled) }
    pub fn invoke(&self, invocation: FunctionInvocation) -> Result<FunctionOutput> {
        let found = self.find(&invocation.function_name);
        if found.is_none() { bail!("versionless function is not registered or disabled: {}", invocation.function_name); }
        Ok(FunctionOutput { function_name: invocation.function_name, ok: true, output: json!({"accepted": true, "mode": "rust-versionless-executor", "transport": "graphql-cosmo", "no_rest": true}), completed_at: Utc::now() })
    }
}
