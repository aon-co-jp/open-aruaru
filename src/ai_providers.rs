//! AI プロバイダー catalog + 用途別おすすめ（開発 / 英会話 / 低単価）

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub vendor: String,
    /// cloud | local | aruaru
    pub kind: String,
    pub models: Vec<String>,
    /// low | medium | high
    pub cost_tier: String,
    pub strengths: Vec<String>,
    pub speech_quality: Option<String>,
    pub translation_quality: Option<String>,
    pub recommended_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoredProvider {
    pub provider: AiProvider,
    pub score: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendUseCase {
    Development,
    EnglishConversation,
    ServerOps,
    Learning,
    Cheap,
    Design,
}

pub fn list_providers() -> Vec<AiProvider> {
    vec![
        AiProvider {
            id: "gpt-4o-mini".into(),
            name: "GPT-4o mini".into(),
            vendor: "OpenAI".into(),
            kind: "cloud".into(),
            models: vec!["gpt-4o-mini".into()],
            cost_tier: "low".into(),
            strengths: vec!["低単価".into(), "英会話".into(), "翻訳".into()],
            speech_quality: Some("high".into()),
            translation_quality: Some("high".into()),
            recommended_for: vec!["english_conversation".into(), "cheap".into()],
        },
        AiProvider {
            id: "claude-opus".into(),
            name: "Claude Opus".into(),
            vendor: "Anthropic".into(),
            kind: "cloud".into(),
            models: vec!["claude-opus-4".into()],
            cost_tier: "high".into(),
            strengths: vec![
                "設計・アーキテクチャ".into(),
                "長文コンテキスト".into(),
                "Rust/Poem レビュー".into(),
            ],
            speech_quality: None,
            translation_quality: Some("high".into()),
            recommended_for: vec!["development".into(), "design".into(), "learning".into()],
        },
        AiProvider {
            id: "chatgpt".into(),
            name: "ChatGPT (GPT-4o)".into(),
            vendor: "OpenAI".into(),
            kind: "cloud".into(),
            models: vec!["gpt-4o".into(), "gpt-4o-mini".into()],
            cost_tier: "medium".into(),
            strengths: vec![
                "コード生成".into(),
                "TypeScript/Vue/React".into(),
                "汎用開発".into(),
            ],
            speech_quality: Some("high".into()),
            translation_quality: Some("high".into()),
            recommended_for: vec!["development".into(), "english_conversation".into()],
        },
        AiProvider {
            id: "gemini".into(),
            name: "Gemini".into(),
            vendor: "Google".into(),
            kind: "cloud".into(),
            models: vec!["gemini-2.0-flash".into(), "gemini-2.5-pro".into()],
            cost_tier: "low".into(),
            strengths: vec![
                "低単価・高速".into(),
                "多言語翻訳".into(),
                "長コンテキスト".into(),
            ],
            speech_quality: Some("medium".into()),
            translation_quality: Some("high".into()),
            recommended_for: vec![
                "cheap".into(),
                "english_conversation".into(),
                "development".into(),
            ],
        },
        AiProvider {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            vendor: "DeepSeek".into(),
            kind: "cloud".into(),
            models: vec!["deepseek-chat".into(), "deepseek-coder".into()],
            cost_tier: "low".into(),
            strengths: vec![
                "低単価コーディング".into(),
                "Rust/TypeScript".into(),
                "API 経由".into(),
            ],
            speech_quality: None,
            translation_quality: Some("medium".into()),
            recommended_for: vec!["cheap".into(), "development".into()],
        },
        AiProvider {
            id: "aruaru-llm".into(),
            name: "aruaru-llm（ローカル）".into(),
            vendor: "aruaru".into(),
            kind: "local".into(),
            models: vec!["aruaru-llm-dev".into()],
            cost_tier: "low".into(),
            strengths: vec![
                "オフライン".into(),
                "DATA 互換".into(),
                "第二の Cursor 連携".into(),
            ],
            speech_quality: Some("medium".into()),
            translation_quality: Some("medium".into()),
            recommended_for: vec!["cheap".into(), "learning".into(), "server_ops".into()],
        },
        AiProvider {
            id: "ollama".into(),
            name: "Ollama（ローカル LLM）".into(),
            vendor: "Ollama".into(),
            kind: "local".into(),
            models: vec!["llama3".into(), "qwen2.5".into(), "mistral".into()],
            cost_tier: "low".into(),
            strengths: vec!["完全ローカル".into(), "プライバシー".into()],
            speech_quality: None,
            translation_quality: Some("medium".into()),
            recommended_for: vec!["cheap".into(), "learning".into()],
        },
    ]
}

pub fn recommend(use_case: RecommendUseCase) -> Vec<AiProvider> {
    recommend_scored(use_case).into_iter().map(|s| s.provider).collect()
}

pub fn recommend_scored(use_case: RecommendUseCase) -> Vec<ScoredProvider> {
    let key = match use_case {
        RecommendUseCase::Development => "development",
        RecommendUseCase::EnglishConversation => "english_conversation",
        RecommendUseCase::ServerOps => "server_ops",
        RecommendUseCase::Learning => "learning",
        RecommendUseCase::Cheap => "cheap",
        RecommendUseCase::Design => "design",
    };
    let mut out: Vec<ScoredProvider> = list_providers()
        .into_iter()
        .filter(|p| p.recommended_for.iter().any(|r| r == key))
        .map(|p| score_provider(&p, &use_case))
        .collect();

    out.sort_by(|a, b| b.score.cmp(&a.score));
    out
}

fn score_provider(p: &AiProvider, use_case: &RecommendUseCase) -> ScoredProvider {
    let mut score: u32 = 0;
    let mut reasons: Vec<String> = Vec::new();

    match use_case {
        RecommendUseCase::Development => {
            score += match p.id.as_str() {
                "claude-opus" => 40,
                "chatgpt" => 35,
                "deepseek" => 30,
                "gemini" => 25,
                _ => 10,
            };
            if p.cost_tier == "low" {
                score += 15;
                reasons.push("低単価".into());
            }
            if p.strengths.iter().any(|s| s.contains("Rust") || s.contains("TypeScript")) {
                score += 10;
                reasons.push("Tauri/Poem 向け".into());
            }
        }
        RecommendUseCase::EnglishConversation => {
            score += match p.speech_quality.as_deref() {
                Some("high") => 25,
                Some("medium") => 15,
                _ => 0,
            };
            score += match p.translation_quality.as_deref() {
                Some("high") => 25,
                Some("medium") => 12,
                _ => 0,
            };
            if p.cost_tier == "low" {
                score += 20;
                reasons.push("低単価".into());
            }
            if p.speech_quality.is_some() {
                reasons.push("音声対応".into());
            }
            if p.translation_quality == Some("high".into()) {
                reasons.push("翻訳高精度".into());
            }
        }
        RecommendUseCase::Cheap => {
            if p.cost_tier == "low" {
                score += 30;
                reasons.push("低単価".into());
            }
        }
        _ => {
            score += 10;
        }
    }

    if reasons.is_empty() {
        reasons.push(p.strengths.first().cloned().unwrap_or_else(|| "汎用".into()));
    }

    ScoredProvider {
        provider: p.clone(),
        score,
        reason: reasons.join(" · "),
    }
}

pub fn stack_for_development() -> serde_json::Value {
    serde_json::json!({
        "name": "aruaru default stack",
        "layers": [
            { "layer": "desktop", "tech": ["Tauri", "Rust", "TypeScript"] },
            { "layer": "mobile", "tech": ["aruaru-android", "aruaru-iPhone", "Tauri Mobile"] },
            { "layer": "web", "tech": ["Poem", "TypeScript", "Vite", "Bootstrap"] },
            { "layer": "api", "tech": ["WunderGraph Cosmo", "VersionLess API"] },
            { "layer": "database", "tech": ["PostgreSQL", "aruaru-db DUAL", "CockroachDB optional"] }
        ],
        "note": "第二の KUSANAGI + 第二の Cursor — DATA 互換 footprint で OS 再インストール後も復元"
    })
}
