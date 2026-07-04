//! aruaru-ai API key handoff planner.
//!
//! This module does not read secrets from browsers or other applications.
//! It only creates a safe import plan for keys that the user explicitly provides
//! by environment variable, manual paste, selected file, or an aruaru-managed vault.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProvider {
    OpenAi,
    AnthropicClaude,
    Gemini,
    DeepSeek,
    LocalLlm,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiKeySource {
    ManualPaste,
    EnvironmentVariable { name: String },
    SelectedEncryptedBackup { path: String },
    AruaruVault { vault_id: String },
    WindowsCredentialManager { target_name: String },
    MacOsKeychain { service_name: String },
    LinuxSecretService { collection_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiKeyTargetStore {
    SessionOnly,
    OperatingSystemSecretStore,
    AruaruEncryptedVault,
    EnvironmentVariable { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyHandoffInput {
    pub provider: AiProvider,
    pub source: ApiKeySource,
    pub target_store: ApiKeyTargetStore,
    pub project_path: String,
    pub allow_persistent_store: bool,
    pub allow_api_test_request: bool,
    pub allow_export: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyHandoffPlan {
    pub provider_label: String,
    pub import_steps: Vec<String>,
    pub safety_checks: Vec<String>,
    pub blocked_actions: Vec<String>,
    pub required_user_confirmations: Vec<String>,
    pub recommended_env_var: Option<String>,
    pub masked_preview_policy: String,
}

impl AiProvider {
    pub fn label(&self) -> String {
        match self {
            AiProvider::OpenAi => "OpenAI / ChatGPT API".to_string(),
            AiProvider::AnthropicClaude => "Anthropic Claude / Opus API".to_string(),
            AiProvider::Gemini => "Google Gemini API".to_string(),
            AiProvider::DeepSeek => "DeepSeek API".to_string(),
            AiProvider::LocalLlm => "Local LLM".to_string(),
            AiProvider::Custom(name) => format!("Custom provider: {name}"),
        }
    }

    pub fn recommended_env_var(&self) -> Option<String> {
        match self {
            AiProvider::OpenAi => Some("OPENAI_API_KEY".to_string()),
            AiProvider::AnthropicClaude => Some("ANTHROPIC_API_KEY".to_string()),
            AiProvider::Gemini => Some("GEMINI_API_KEY".to_string()),
            AiProvider::DeepSeek => Some("DEEPSEEK_API_KEY".to_string()),
            AiProvider::LocalLlm => None,
            AiProvider::Custom(_) => Some("ARUARU_CUSTOM_AI_API_KEY".to_string()),
        }
    }
}

pub fn mask_secret(secret: &str) -> String {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return "<empty>".to_string();
    }

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= 8 {
        return "********".to_string();
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

pub fn build_api_key_handoff_plan(input: &ApiKeyHandoffInput) -> ApiKeyHandoffPlan {
    let provider_label = input.provider.label();
    let recommended_env_var = input.provider.recommended_env_var();

    let mut import_steps = vec![
        "Confirm the project directory and provider selection.".to_string(),
        "Import only a key that the user explicitly owns or is authorized to use.".to_string(),
        "Never print the full key to logs, browser UI, README files, or generated scripts."
            .to_string(),
    ];

    match &input.source {
        ApiKeySource::ManualPaste => import_steps
            .push("Open a masked input field and accept manual paste from the user.".to_string()),
        ApiKeySource::EnvironmentVariable { name } => import_steps.push(format!(
            "Read key from environment variable `{name}` after user approval."
        )),
        ApiKeySource::SelectedEncryptedBackup { path } => import_steps.push(format!(
            "Ask the user to select and decrypt backup file `{path}`."
        )),
        ApiKeySource::AruaruVault { vault_id } => {
            import_steps.push(format!("Import from aruaru encrypted vault `{vault_id}`."))
        }
        ApiKeySource::WindowsCredentialManager { target_name } => import_steps.push(format!(
            "Import from Windows Credential Manager target `{target_name}`."
        )),
        ApiKeySource::MacOsKeychain { service_name } => import_steps.push(format!(
            "Import from macOS Keychain service `{service_name}`."
        )),
        ApiKeySource::LinuxSecretService { collection_name } => import_steps.push(format!(
            "Import from Linux Secret Service collection `{collection_name}`."
        )),
    }

    match &input.target_store {
        ApiKeyTargetStore::SessionOnly => import_steps
            .push("Keep the key only in memory for the current aruaru-ai session.".to_string()),
        ApiKeyTargetStore::OperatingSystemSecretStore => import_steps
            .push("Store through the operating system secret store when supported.".to_string()),
        ApiKeyTargetStore::AruaruEncryptedVault => import_steps.push(
            "Store in aruaru encrypted vault with passphrase or OS-backed encryption.".to_string(),
        ),
        ApiKeyTargetStore::EnvironmentVariable { name } => import_steps.push(format!(
            "Write to user-level environment variable `{name}` only after confirmation."
        )),
    }

    let mut safety_checks = vec![
        "Mask key preview, for example sk-...abcd or ant-...wxyz.".to_string(),
        "Reject accidental paste into normal chat messages when possible.".to_string(),
        "Check .gitignore contains .env, .aruaru/secrets, and local vault files.".to_string(),
        "Scan generated logs and scripts for leaked key patterns before saving.".to_string(),
        "Run a minimal provider health check only if the user allows an API test request."
            .to_string(),
        "Save provider, key alias, creation time, and permission scope, but not the plaintext key."
            .to_string(),
    ];

    if !input.allow_api_test_request {
        safety_checks
            .push("Skip live API validation because API test request is disabled.".to_string());
    }

    let mut blocked_actions = vec![
        "Do not search browser storage, password managers, or unrelated app folders automatically."
            .to_string(),
        "Do not import another person's key without explicit authorization.".to_string(),
        "Do not commit API keys to Git.".to_string(),
        "Do not embed keys into README.md, README.html, README.rs, TypeScript, or frontend code."
            .to_string(),
        "Do not show the full key after import.".to_string(),
        "Do not export plaintext keys by default.".to_string(),
    ];

    if !input.allow_export {
        blocked_actions.push("Plaintext export is disabled for this handoff plan.".to_string());
    }

    let mut required_user_confirmations = vec![
        "I own this API key or have permission to use it.".to_string(),
        "I understand paid API usage may incur charges.".to_string(),
        "I approve storing or using this key for the selected project.".to_string(),
    ];

    if input.allow_persistent_store {
        required_user_confirmations
            .push("I approve persistent encrypted storage of this key.".to_string());
    } else {
        required_user_confirmations
            .push("Persistent storage is disabled; the key will be session-only.".to_string());
    }

    ApiKeyHandoffPlan {
        provider_label,
        import_steps,
        safety_checks,
        blocked_actions,
        required_user_confirmations,
        recommended_env_var,
        masked_preview_policy: "Always display only first 4 and last 4 characters; never display full plaintext after import.".to_string(),
    }
}

/// Small runtime integration hook used by aruaru-desktop/aruaru-ai quality gates.
///
/// This keeps the API key handoff feature connected to the binary while the UI
/// wiring is still being developed. It never reads, writes, logs, or exports a
/// real API key.
pub fn quality_gate_smoke_check() -> bool {
    let providers = [
        AiProvider::OpenAi,
        AiProvider::AnthropicClaude,
        AiProvider::Gemini,
        AiProvider::DeepSeek,
        AiProvider::LocalLlm,
        AiProvider::Custom("custom".to_string()),
    ];

    let sources = [
        ApiKeySource::ManualPaste,
        ApiKeySource::EnvironmentVariable {
            name: "OPENAI_API_KEY".to_string(),
        },
        ApiKeySource::SelectedEncryptedBackup {
            path: ".aruaru/secrets/backup.age".to_string(),
        },
        ApiKeySource::AruaruVault {
            vault_id: "default".to_string(),
        },
        ApiKeySource::WindowsCredentialManager {
            target_name: "aruaru/openai".to_string(),
        },
        ApiKeySource::MacOsKeychain {
            service_name: "aruaru".to_string(),
        },
        ApiKeySource::LinuxSecretService {
            collection_name: "aruaru".to_string(),
        },
    ];

    let target_stores = [
        ApiKeyTargetStore::SessionOnly,
        ApiKeyTargetStore::OperatingSystemSecretStore,
        ApiKeyTargetStore::AruaruEncryptedVault,
        ApiKeyTargetStore::EnvironmentVariable {
            name: "OPENAI_API_KEY".to_string(),
        },
    ];

    let input = ApiKeyHandoffInput {
        provider: providers[0].clone(),
        source: sources[0].clone(),
        target_store: target_stores[0].clone(),
        project_path: ".".to_string(),
        allow_persistent_store: false,
        allow_api_test_request: false,
        allow_export: false,
    };
    let plan = build_api_key_handoff_plan(&input);

    !providers.is_empty()
        && !sources.is_empty()
        && !target_stores.is_empty()
        && !plan.import_steps.is_empty()
        && mask_secret("sk-test-secret-value").contains("...")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_short_secret() {
        assert_eq!(mask_secret("abc"), "********");
    }

    #[test]
    fn masks_long_secret() {
        assert_eq!(mask_secret("sk-1234567890abcd"), "sk-1...abcd");
    }

    #[test]
    fn openai_env_var_is_recommended() {
        assert_eq!(
            AiProvider::OpenAi.recommended_env_var(),
            Some("OPENAI_API_KEY".to_string())
        );
    }

    #[test]
    fn plan_blocks_unsafe_imports() {
        let input = ApiKeyHandoffInput {
            provider: AiProvider::AnthropicClaude,
            source: ApiKeySource::ManualPaste,
            target_store: ApiKeyTargetStore::SessionOnly,
            project_path: "F:\\aruaru\\aruaru-rs4".to_string(),
            allow_persistent_store: false,
            allow_api_test_request: false,
            allow_export: false,
        };
        let plan = build_api_key_handoff_plan(&input);
        assert!(plan.provider_label.contains("Claude"));
        assert!(plan
            .blocked_actions
            .iter()
            .any(|line| line.contains("another person's key")));
        assert!(plan
            .required_user_confirmations
            .iter()
            .any(|line| line.contains("permission")));
    }
}
