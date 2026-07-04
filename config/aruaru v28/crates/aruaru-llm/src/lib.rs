use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider { ChatGpt, ClaudeOpus, AruaruLlm, ILumiLlm, VpsWorker }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProfile {
    pub provider: LlmProvider,
    pub model_name: String,
    pub local_gpu_required: bool,
    pub low_power_mode: bool,
    pub notes: String,
}

impl LlmProfile {
    pub fn gt730_safe_default() -> Self {
        Self {
            provider: LlmProvider::VpsWorker,
            model_name: "aruaru-llm-light-or-cloud".to_string(),
            local_gpu_required: false,
            low_power_mode: true,
            notes: "GT730 PCではローカルGPU LLMを要求せず、ChatGPT/Claude/VPS/aruaru-llm軽量workerへ逃がす".to_string(),
        }
    }
    pub fn by_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "chatgpt" => Self { provider: LlmProvider::ChatGpt, model_name: "ChatGPT selectable".to_string(), local_gpu_required: false, low_power_mode: true, notes: "external AI provider".to_string() },
            "opus" | "claude" | "claude-opus" => Self { provider: LlmProvider::ClaudeOpus, model_name: "Claude Opus selectable".to_string(), local_gpu_required: false, low_power_mode: true, notes: "external AI provider".to_string() },
            "ilumi" | "ilumi-llm" => Self { provider: LlmProvider::ILumiLlm, model_name: "iLumi-llm".to_string(), local_gpu_required: false, low_power_mode: true, notes: "local/VPS hybrid selectable".to_string() },
            "aruaru" | "aruaru-llm" => Self { provider: LlmProvider::AruaruLlm, model_name: "aruaru-llm".to_string(), local_gpu_required: false, low_power_mode: true, notes: "project native LLM".to_string() },
            _ => Self::gt730_safe_default(),
        }
    }
}
