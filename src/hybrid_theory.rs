#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::llm_folding::{build_deepseek_folding_plan, FoldingInput, FoldingMode};
use crate::quasi_quantum::{build_toshiba_sbm_plan, OptimizationTarget, ToshibaSbmPlanInput};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HybridBudgetMode {
    VolunteerGt730,
    LowCostSingleGpu,
    HighVramSingleGpu,
    ApiAssisted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HybridExecutionPolicy {
    OfflineFirst,
    LocalFirstWithApiFallback,
    ApiForHardCasesOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HybridPlannerInput {
    pub gpu_name: String,
    pub vram_gb: u32,
    pub system_ram_gb: u32,
    pub monthly_ai_budget_yen: u32,
    pub project_is_volunteer: bool,
    pub allow_api_fallback: bool,
    pub target: HybridTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HybridTarget {
    AutoBugCheck,
    PatchPlanning,
    ReadmeGeneration,
    LlmModelSelection,
    LongLogAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HybridAction {
    pub name: String,
    pub purpose: String,
    pub cheap_first: bool,
    pub requires_user_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HybridFoldingSbmPlan {
    pub name: String,
    pub budget_mode: HybridBudgetMode,
    pub execution_policy: HybridExecutionPolicy,
    pub deepseek_folding_summary: String,
    pub sbm_optimizer_summary: String,
    pub fusion_strategy: String,
    pub local_model_policy: String,
    pub api_policy: String,
    pub privacy_policy: String,
    pub volunteer_note: String,
    pub safety_note: String,
    pub actions: Vec<HybridAction>,
}

/// Build the integrated aruaru-llm planner.
///
/// The planner fuses two practical ideas:
/// 1. DeepSeek-inspired folding: distillation, quantization, MoE-style routing,
///    sparse/context folding, and API fallback.
/// 2. Toshiba SBM-inspired optimization: QUBO-like ordering for tests, fixes,
///    model routes, and prompt fragments.
///
/// This is intentionally honest: it does not claim that a GT730 or one consumer
/// GPU can run a 671B frontier model at full quality. It turns the public ideas
/// into a low-cost engineering strategy for aruaru bug checking.
pub fn build_hybrid_folding_sbm_plan(input: &HybridPlannerInput) -> HybridFoldingSbmPlan {
    let gt730_like = input.vram_gb < 4 || input.gpu_name.to_lowercase().contains("gt730");
    let budget_mode = if gt730_like || input.project_is_volunteer || input.monthly_ai_budget_yen <= 1_000 {
        HybridBudgetMode::VolunteerGt730
    } else if input.vram_gb >= 24 {
        HybridBudgetMode::HighVramSingleGpu
    } else if input.allow_api_fallback {
        HybridBudgetMode::ApiAssisted
    } else {
        HybridBudgetMode::LowCostSingleGpu
    };

    let execution_policy = if !input.allow_api_fallback {
        HybridExecutionPolicy::OfflineFirst
    } else if matches!(budget_mode, HybridBudgetMode::VolunteerGt730) {
        HybridExecutionPolicy::ApiForHardCasesOnly
    } else {
        HybridExecutionPolicy::LocalFirstWithApiFallback
    };

    let folding_mode = match input.target {
        HybridTarget::AutoBugCheck | HybridTarget::PatchPlanning => FoldingMode::BugCheck,
        HybridTarget::ReadmeGeneration => FoldingMode::ReadmeGeneration,
        HybridTarget::LlmModelSelection => FoldingMode::CodeReview,
        HybridTarget::LongLogAnalysis => FoldingMode::LongContextResearch,
    };

    let deepseek_plan = build_deepseek_folding_plan(&FoldingInput {
        gpu_name: input.gpu_name.clone(),
        vram_gb: input.vram_gb,
        system_ram_gb: input.system_ram_gb,
        mode: folding_mode,
        prefer_offline: !input.allow_api_fallback,
    });

    let optimization_target = match input.target {
        HybridTarget::AutoBugCheck | HybridTarget::PatchPlanning => OptimizationTarget::BugFixOrder,
        HybridTarget::ReadmeGeneration => OptimizationTarget::ReadmeGenerationPlan,
        HybridTarget::LlmModelSelection => OptimizationTarget::LlmModelRoute,
        HybridTarget::LongLogAnalysis => OptimizationTarget::PromptCompression,
    };

    let variable_count = match input.target {
        HybridTarget::AutoBugCheck => 128,
        HybridTarget::PatchPlanning => 256,
        HybridTarget::ReadmeGeneration => 96,
        HybridTarget::LlmModelSelection => 64,
        HybridTarget::LongLogAnalysis => 512,
    };

    let sbm_plan = build_toshiba_sbm_plan(&ToshibaSbmPlanInput {
        gpu_name: input.gpu_name.clone(),
        vram_gb: input.vram_gb,
        variables: variable_count,
        target: optimization_target,
        need_accuracy: true,
        allow_external_sqbm: false,
    });

    let local_model_policy = match budget_mode {
        HybridBudgetMode::VolunteerGt730 => {
            "GT730 mode: do not chase huge local models; use 1.5B-7B class, CPU/hybrid execution, short context, log folding, and scripted checks first".to_string()
        }
        HybridBudgetMode::LowCostSingleGpu => {
            "Low-cost single GPU mode: use 4-bit 7B-14B class models, keep context small, and route only hard spans to stronger models".to_string()
        }
        HybridBudgetMode::HighVramSingleGpu => {
            "High-VRAM single GPU mode: use 14B-32B class local models with quantization, KV/context folding, and larger bug-fix batches".to_string()
        }
        HybridBudgetMode::ApiAssisted => {
            "API-assisted mode: use local models for triage and paid API models only for unresolved compile/design errors".to_string()
        }
    };

    let api_policy = match execution_policy {
        HybridExecutionPolicy::OfflineFirst => {
            "API disabled: all checks stay local; use manual scripts and small local models".to_string()
        }
        HybridExecutionPolicy::ApiForHardCasesOnly => {
            "API is used only after cheap local checks fail; send minimized, redacted error cards instead of whole projects".to_string()
        }
        HybridExecutionPolicy::LocalFirstWithApiFallback => {
            "Local-first with API fallback: use Opus/ChatGPT/etc. for high-value fixes after filtering secrets and large generated folders".to_string()
        }
    };

    let fusion_strategy = "DeepSeek folding reduces information cost; Toshiba SBM-inspired QUBO planning chooses the cheapest useful next action; ordinary cargo/PowerShell quality gates verify the result".to_string();

    let mut actions = vec![
        HybridAction {
            name: "collect_evidence".to_string(),
            purpose: "Collect Cargo.toml, failing logs, touched Rust files, README deltas, and check scripts".to_string(),
            cheap_first: true,
            requires_user_approval: false,
        },
        HybridAction {
            name: "fold_context".to_string(),
            purpose: "Fold long logs into structured error cards before using any expensive LLM".to_string(),
            cheap_first: true,
            requires_user_approval: false,
        },
        HybridAction {
            name: "optimize_order".to_string(),
            purpose: "Use SBM-inspired scoring to choose fix/test/model order under a low-cost budget".to_string(),
            cheap_first: true,
            requires_user_approval: false,
        },
        HybridAction {
            name: "propose_patch".to_string(),
            purpose: "Generate a patch proposal and diff without overwriting files".to_string(),
            cheap_first: false,
            requires_user_approval: true,
        },
        HybridAction {
            name: "verify_quality_gate".to_string(),
            purpose: "Run fmt/check/test/clippy and banned-feature checks after approved changes".to_string(),
            cheap_first: true,
            requires_user_approval: false,
        },
    ];

    if input.allow_api_fallback {
        actions.insert(3, HybridAction {
            name: "api_hard_case_review".to_string(),
            purpose: "Use a paid API model only for unresolved hard cases after redaction and context minimization".to_string(),
            cheap_first: false,
            requires_user_approval: true,
        });
    }

    HybridFoldingSbmPlan {
        name: "aruaru-llm Hybrid Folding + SBM-Inspired Planner".to_string(),
        budget_mode,
        execution_policy,
        deepseek_folding_summary: format!(
            "{} | {} | {}",
            deepseek_plan.model_tier, deepseek_plan.quantization, deepseek_plan.context_strategy
        ),
        sbm_optimizer_summary: format!(
            "{} | {} | {}",
            sbm_plan.aruaru_use_case, sbm_plan.parameter_strategy, sbm_plan.hardware_strategy
        ),
        fusion_strategy,
        local_model_policy,
        api_policy,
        privacy_policy: "Never send .env, API keys, SSH keys, target/, node_modules/, .git/, or unrelated private files to API LLMs".to_string(),
        volunteer_note: "Designed for a no-salary volunteer development context where even AI fees, domain renewals, and VPS costs may be paid personally; therefore cheap local checks run before paid API calls".to_string(),
        safety_note: "This is an experimental planner, not proof that a GT730 or one GPU can fully replace a 100k-GPU supercomputer, Toshiba SQBM+, Fujitsu quantum hardware, or paid frontier-model infrastructure".to_string(),
        actions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gt730_volunteer_uses_api_only_for_hard_cases() {
        let plan = build_hybrid_folding_sbm_plan(&HybridPlannerInput {
            gpu_name: "NVIDIA GT730".to_string(),
            vram_gb: 2,
            system_ram_gb: 16,
            monthly_ai_budget_yen: 0,
            project_is_volunteer: true,
            allow_api_fallback: true,
            target: HybridTarget::AutoBugCheck,
        });
        assert_eq!(plan.budget_mode, HybridBudgetMode::VolunteerGt730);
        assert_eq!(plan.execution_policy, HybridExecutionPolicy::ApiForHardCasesOnly);
        assert!(plan.local_model_policy.contains("1.5B-7B"));
        assert!(plan.privacy_policy.contains("API keys"));
    }

    #[test]
    fn no_api_mode_stays_offline_first() {
        let plan = build_hybrid_folding_sbm_plan(&HybridPlannerInput {
            gpu_name: "NVIDIA RTX 3060".to_string(),
            vram_gb: 12,
            system_ram_gb: 64,
            monthly_ai_budget_yen: 0,
            project_is_volunteer: false,
            allow_api_fallback: false,
            target: HybridTarget::LongLogAnalysis,
        });
        assert_eq!(plan.execution_policy, HybridExecutionPolicy::OfflineFirst);
        assert!(plan.api_policy.contains("API disabled"));
        assert!(plan.fusion_strategy.contains("DeepSeek folding"));
    }
}
