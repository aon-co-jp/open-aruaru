#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FoldingMode {
    BugCheck,
    CodeReview,
    ReadmeGeneration,
    LongContextResearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackendKind {
    LocalGpu,
    LocalCpu,
    ApiFallback,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoldingInput {
    pub gpu_name: String,
    pub vram_gb: u32,
    pub system_ram_gb: u32,
    pub mode: FoldingMode,
    pub prefer_offline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoldingPlan {
    pub backend: BackendKind,
    pub model_tier: String,
    pub quantization: String,
    pub attention_strategy: String,
    pub routing_strategy: String,
    pub context_strategy: String,
    pub safety_note: String,
    pub steps: Vec<String>,
}

/// DeepSeek-inspired local inference planner.
///
/// This does not claim that one consumer GPU can reproduce a full 671B-parameter
/// frontier model. It turns the public DeepSeek ideas into practical local steps:
/// MoE-style routing, MLA/KV-cache compression, sparse/selected context, distilled
/// reasoning models, quantization, and API fallback when local hardware is too small.
pub fn build_deepseek_folding_plan(input: &FoldingInput) -> FoldingPlan {
    let low_gpu = input.vram_gb < 8 || input.gpu_name.to_lowercase().contains("gt730");
    let backend = if input.prefer_offline {
        if low_gpu { BackendKind::LocalCpu } else { BackendKind::LocalGpu }
    } else if low_gpu {
        BackendKind::Hybrid
    } else {
        BackendKind::LocalGpu
    };

    let model_tier = match input.vram_gb {
        0..=7 => "DeepSeek-R1-Distill 1.5B-7B / small code model".to_string(),
        8..=15 => "DeepSeek-R1-Distill 7B-14B class".to_string(),
        16..=31 => "DeepSeek-R1-Distill 14B-32B class".to_string(),
        32..=63 => "DeepSeek-R1-Distill 32B-70B class with offload".to_string(),
        _ => "large local MoE or 70B+ class with sharding/offload".to_string(),
    };

    let quantization = match input.vram_gb {
        0..=7 => "Q4_K_M or smaller; CPU fallback; short context".to_string(),
        8..=15 => "4-bit quantization; moderate context".to_string(),
        16..=31 => "4-bit/5-bit quantization; larger context".to_string(),
        _ => "4-bit/8-bit mixed quantization; optional KV cache quantization".to_string(),
    };

    let attention_strategy = match input.mode {
        FoldingMode::LongContextResearch => {
            "Native Sparse Attention style: coarse token compression + fine token selection + local window".to_string()
        }
        _ => "MLA/KV folding style: compress repeated context, keep recent error lines dense".to_string(),
    };

    let routing_strategy = match input.mode {
        FoldingMode::BugCheck | FoldingMode::CodeReview => {
            "expert folding: route compiler errors, file diffs, README generation, and policy checks to separate lightweight experts".to_string()
        }
        FoldingMode::ReadmeGeneration => {
            "document folding: route headings, code blocks, tables, and responsive HTML generation separately".to_string()
        }
        FoldingMode::LongContextResearch => {
            "context folding: route summary blocks, evidence lines, and final synthesis separately".to_string()
        }
    };

    let context_strategy = if low_gpu {
        "GT730/small GPU mode: summarize logs first, then send only failing spans to the model".to_string()
    } else {
        "single GPU mode: keep hot context in VRAM, fold cold context into summaries, rehydrate files on demand".to_string()
    };

    let mut steps = vec![
        "scan project and collect Cargo/PowerShell/README evidence".to_string(),
        "fold long logs into structured error cards".to_string(),
        "select local model tier from VRAM and RAM".to_string(),
        "apply quantization and KV/context folding plan".to_string(),
        "generate patch proposal, not direct overwrite".to_string(),
        "run cargo fmt/check/test/clippy and banned-feature checks".to_string(),
    ];

    if !input.prefer_offline {
        steps.push("use API fallback for errors that exceed local model capability".to_string());
    }

    FoldingPlan {
        backend,
        model_tier,
        quantization,
        attention_strategy,
        routing_strategy,
        context_strategy,
        safety_note: "This is an experimental DeepSeek-inspired folding planner. It does not guarantee frontier-model quality on one GPU.".to_string(),
        steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gt730_uses_cpu_or_hybrid_and_small_model() {
        let plan = build_deepseek_folding_plan(&FoldingInput {
            gpu_name: "NVIDIA GT730".to_string(),
            vram_gb: 2,
            system_ram_gb: 16,
            mode: FoldingMode::BugCheck,
            prefer_offline: false,
        });
        assert_eq!(plan.backend, BackendKind::Hybrid);
        assert!(plan.model_tier.contains("1.5B-7B"));
        assert!(plan.context_strategy.contains("GT730"));
    }

    #[test]
    fn twenty_four_gb_gpu_uses_mid_size_distill() {
        let plan = build_deepseek_folding_plan(&FoldingInput {
            gpu_name: "NVIDIA RTX 3090".to_string(),
            vram_gb: 24,
            system_ram_gb: 128,
            mode: FoldingMode::CodeReview,
            prefer_offline: true,
        });
        assert_eq!(plan.backend, BackendKind::LocalGpu);
        assert!(plan.model_tier.contains("14B-32B"));
        assert!(plan.routing_strategy.contains("expert folding"));
    }
}
