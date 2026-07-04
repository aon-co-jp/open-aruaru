#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SbmGeneration {
    Ballistic,
    Discrete,
    EdgeOfChaos,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SbmBackend {
    CpuSafe,
    SingleGpu,
    MultiGpu,
    ExternalSqbm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptimizationTarget {
    BugFixOrder,
    TestOrder,
    LlmModelRoute,
    PromptCompression,
    ReadmeGenerationPlan,
    GenericQubo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToshibaSbmPlanInput {
    pub gpu_name: String,
    pub vram_gb: u32,
    pub variables: usize,
    pub target: OptimizationTarget,
    pub need_accuracy: bool,
    pub allow_external_sqbm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToshibaSbmPlan {
    pub generation: SbmGeneration,
    pub backend: SbmBackend,
    pub qubo_strategy: String,
    pub parameter_strategy: String,
    pub hardware_strategy: String,
    pub aruaru_use_case: String,
    pub safety_note: String,
    pub steps: Vec<String>,
}

/// Build an aruaru-llm planning policy inspired by Toshiba's Simulated
/// Bifurcation Machine (SBM) / SQBM+ public materials.
///
/// This is not Toshiba SQBM+ itself and does not claim quantum-computer
/// performance. It maps aruaru tasks into QUBO-like binary optimization plans
/// and chooses a CPU/GPU-safe heuristic path for ordinary PCs.
pub fn build_toshiba_sbm_plan(input: &ToshibaSbmPlanInput) -> ToshibaSbmPlan {
    let small_gpu = input.vram_gb < 8 || input.gpu_name.to_lowercase().contains("gt730");
    let large_problem = input.variables >= 100_000;

    let backend = if input.allow_external_sqbm && large_problem {
        SbmBackend::ExternalSqbm
    } else if small_gpu {
        SbmBackend::CpuSafe
    } else if input.vram_gb >= 24 {
        SbmBackend::SingleGpu
    } else {
        SbmBackend::CpuSafe
    };

    let generation = if input.need_accuracy {
        if input.variables >= 2_000 {
            SbmGeneration::EdgeOfChaos
        } else {
            SbmGeneration::Discrete
        }
    } else {
        SbmGeneration::Ballistic
    };

    let qubo_strategy = match input.target {
        OptimizationTarget::BugFixOrder => {
            "convert candidate fixes into binary variables; penalize risky order, reward fixes that unblock cargo check first".to_string()
        }
        OptimizationTarget::TestOrder => {
            "convert tests into binary/order buckets; prioritize fast failing checks before expensive integration checks".to_string()
        }
        OptimizationTarget::LlmModelRoute => {
            "convert model choices into route variables; balance cost, latency, privacy, and expected repair accuracy".to_string()
        }
        OptimizationTarget::PromptCompression => {
            "convert context fragments into keep/drop variables; preserve compiler errors, touched code, and README deltas".to_string()
        }
        OptimizationTarget::ReadmeGenerationPlan => {
            "convert sections and assets into layout variables; keep Markdown/HTML parity and responsive display constraints".to_string()
        }
        OptimizationTarget::GenericQubo => {
            "accept a QUBO matrix directly and run the local experimental solver or an external SQBM-compatible adapter".to_string()
        }
    };

    let parameter_strategy = match generation {
        SbmGeneration::Ballistic => {
            "bSB-like fast mode: fewer shots, shorter schedule, early-stop when a good solution appears".to_string()
        }
        SbmGeneration::Discrete => {
            "dSB-like accuracy mode: discrete spin projection, more shots, stronger local refinement".to_string()
        }
        SbmGeneration::EdgeOfChaos => {
            "third-generation-inspired mode: per-variable bifurcation pressure and edge-of-chaos perturbation to escape local optima".to_string()
        }
    };

    let hardware_strategy = match backend {
        SbmBackend::CpuSafe => {
            "ordinary PC / GT730 mode: use small QUBO windows, deterministic multi-shot CPU search, and avoid VRAM-heavy kernels".to_string()
        }
        SbmBackend::SingleGpu => {
            "single GPU mode: batch many shots and parallelize energy-delta evaluation when a GPU backend is available".to_string()
        }
        SbmBackend::MultiGpu => {
            "multi GPU mode: shard shots and QUBO blocks; reserved for future cluster use".to_string()
        }
        SbmBackend::ExternalSqbm => {
            "external SQBM adapter mode: export QUBO and import the best solution while keeping aruaru approval gates".to_string()
        }
    };

    let aruaru_use_case = match input.target {
        OptimizationTarget::BugFixOrder => "automatic bug-check repair ordering".to_string(),
        OptimizationTarget::TestOrder => "quality-gate test scheduling".to_string(),
        OptimizationTarget::LlmModelRoute => "aruaru-llm model selection".to_string(),
        OptimizationTarget::PromptCompression => "DeepSeek folding context selection".to_string(),
        OptimizationTarget::ReadmeGenerationPlan => "README.md / README.rs / README.html generation planning".to_string(),
        OptimizationTarget::GenericQubo => "generic QUBO experimentation".to_string(),
    };

    ToshibaSbmPlan {
        generation,
        backend,
        qubo_strategy,
        parameter_strategy,
        hardware_strategy,
        aruaru_use_case,
        safety_note: "Experimental quantum-inspired optimizer. It does not claim to be a Toshiba SQBM+ implementation, a quantum computer, or a guaranteed 100x faster Fujitsu quantum computer replacement.".to_string(),
        steps: vec![
            "collect bug-check, LLM, README, or test candidates".to_string(),
            "formulate the choice problem as a small or windowed QUBO".to_string(),
            "choose bSB/dSB/edge-of-chaos-inspired mode from speed and accuracy needs".to_string(),
            "run local deterministic multi-shot search or export to an external SQBM adapter".to_string(),
            "verify result with ordinary cargo/PowerShell quality gates".to_string(),
            "show diff and require user approval before applying changes".to_string(),
        ],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuboProblem {
    pub linear: Vec<f64>,
    pub quadratic: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalSbmConfig {
    pub shots: usize,
    pub steps: usize,
    pub chaos_gain: f64,
    pub prefer_accuracy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalSbmSolution {
    pub bits: Vec<u8>,
    pub energy: f64,
    pub algorithm: SbmGeneration,
    pub iterations: usize,
}

impl QuboProblem {
    pub fn validate(&self) -> Result<(), String> {
        let n = self.linear.len();
        if n == 0 {
            return Err("QUBO problem must contain at least one variable".to_string());
        }
        if self.quadratic.len() != n {
            return Err("quadratic matrix row count must match linear variable count".to_string());
        }
        for row in &self.quadratic {
            if row.len() != n {
                return Err("quadratic matrix must be square".to_string());
            }
        }
        Ok(())
    }

    pub fn energy(&self, bits: &[u8]) -> f64 {
        let mut energy = 0.0;
        for (i, &bit) in bits.iter().enumerate() {
            let xi = f64::from(bit);
            energy += self.linear[i] * xi;
            for (j, &other) in bits.iter().enumerate() {
                energy += 0.5 * self.quadratic[i][j] * xi * f64::from(other);
            }
        }
        energy
    }
}

/// Small deterministic QUBO solver used by aruaru-llm tests and low-end PCs.
///
/// It is inspired by SBM ideas: many shots, parallel-flip style pressure,
/// discrete projection, and edge-of-chaos perturbation. It is intentionally
/// conservative and suitable for small/windowed QUBO subproblems rather than
/// huge production optimization.
pub fn solve_qubo_local(problem: &QuboProblem, config: &LocalSbmConfig) -> Result<LocalSbmSolution, String> {
    problem.validate()?;
    let n = problem.linear.len();
    let shots = config.shots.max(1);
    let steps = config.steps.max(1);
    let mut best_bits = vec![0_u8; n];
    let mut best_energy = problem.energy(&best_bits);

    for shot in 0..shots {
        let mut bits: Vec<u8> = (0..n)
            .map(|i| if ((i + shot) % 3) == 0 { 1 } else { 0 })
            .collect();
        let mut current = problem.energy(&bits);

        for step in 0..steps {
            let pressure = (step as f64 + 1.0) / steps as f64;
            let chaos = config.chaos_gain * ((((shot + 1) * (step + 3)) % 17) as f64 / 17.0 - 0.5);
            let mut changed = false;

            for i in 0..n {
                bits[i] ^= 1;
                let trial = problem.energy(&bits);
                let delta = trial - current;
                let accept = if config.prefer_accuracy {
                    delta < chaos * (1.0 - pressure)
                } else {
                    delta < 0.05 * (1.0 - pressure) + chaos
                };

                if accept {
                    current = trial;
                    changed = true;
                } else {
                    bits[i] ^= 1;
                }
            }

            if current < best_energy {
                best_energy = current;
                best_bits.clone_from(&bits);
            }

            if !changed && config.prefer_accuracy {
                break;
            }
        }
    }

    let algorithm = if config.chaos_gain > 0.0 {
        SbmGeneration::EdgeOfChaos
    } else if config.prefer_accuracy {
        SbmGeneration::Discrete
    } else {
        SbmGeneration::Ballistic
    };

    Ok(LocalSbmSolution {
        bits: best_bits,
        energy: best_energy,
        algorithm,
        iterations: shots * steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gt730_plan_uses_cpu_safe_backend() {
        let plan = build_toshiba_sbm_plan(&ToshibaSbmPlanInput {
            gpu_name: "NVIDIA GT730".to_string(),
            vram_gb: 2,
            variables: 128,
            target: OptimizationTarget::BugFixOrder,
            need_accuracy: true,
            allow_external_sqbm: false,
        });
        assert_eq!(plan.backend, SbmBackend::CpuSafe);
        assert!(plan.hardware_strategy.contains("GT730"));
        assert!(plan.safety_note.contains("Experimental"));
    }

    #[test]
    fn large_problem_can_select_external_sqbm_adapter() {
        let plan = build_toshiba_sbm_plan(&ToshibaSbmPlanInput {
            gpu_name: "NVIDIA RTX 4090".to_string(),
            vram_gb: 24,
            variables: 1_000_000,
            target: OptimizationTarget::GenericQubo,
            need_accuracy: true,
            allow_external_sqbm: true,
        });
        assert_eq!(plan.backend, SbmBackend::ExternalSqbm);
        assert_eq!(plan.generation, SbmGeneration::EdgeOfChaos);
    }

    #[test]
    fn local_solver_finds_low_energy_choice() {
        let problem = QuboProblem {
            linear: vec![-2.0, 1.0],
            quadratic: vec![vec![0.0, 0.0], vec![0.0, 0.0]],
        };
        let solution = solve_qubo_local(&problem, &LocalSbmConfig {
            shots: 4,
            steps: 8,
            chaos_gain: 0.05,
            prefer_accuracy: true,
        }).expect("solver should work");
        assert_eq!(solution.bits[0], 1);
        assert_eq!(solution.bits[1], 0);
        assert!(solution.energy <= -2.0);
    }
}
