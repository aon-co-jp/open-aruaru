//! aruaru-ai README output menu planner.
//!
//! README.md is the fixed source/output document. Additional README.* files are
//! optional checkbox targets. The menu allows selecting none, one, or many.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReadmeOutputTarget {
    Rs,
    Html,
    Php,
    Python,
    TypeScript,
    JavaScript,
    Go,
    Java,
    CSharp,
    Kotlin,
    Swift,
    Ruby,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadmeOutputMenuItem {
    pub target: ReadmeOutputTarget,
    pub checkbox_label: String,
    pub file_name: String,
    pub description: String,
    pub selected_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadmeOutputMenuPlan {
    pub fixed_output: String,
    pub selected_outputs: Vec<String>,
    pub menu_items: Vec<ReadmeOutputMenuItem>,
    pub safety_rules: Vec<String>,
}

impl ReadmeOutputTarget {
    pub fn parse(value: &str) -> Result<Self> {
        let lowered = value.trim().to_ascii_lowercase();
        let normalized = lowered
            .strip_prefix("readme")
            .unwrap_or(&lowered)
            .trim_start_matches('.')
            .replace('-', "")
            .replace('_', "")
            .replace(' ', "");

        match normalized.as_str() {
            "rs" | "rust" => Ok(Self::Rs),
            "html" | "html5" => Ok(Self::Html),
            "php" => Ok(Self::Php),
            "py" | "python" => Ok(Self::Python),
            "ts" | "typescript" => Ok(Self::TypeScript),
            "js" | "javascript" => Ok(Self::JavaScript),
            "go" | "golang" => Ok(Self::Go),
            "java" => Ok(Self::Java),
            "cs" | "csharp" | "c#" => Ok(Self::CSharp),
            "kt" | "kotlin" => Ok(Self::Kotlin),
            "swift" => Ok(Self::Swift),
            "rb" | "ruby" => Ok(Self::Ruby),
            "json" => Ok(Self::Json),
            "" | "none" | "no" | "off" => Err(anyhow!("empty README output target")),
            other => Err(anyhow!("unknown README output target: {other}")),
        }
    }

    pub fn parse_csv(value: &str) -> Result<Vec<Self>> {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || matches!(trimmed.to_ascii_lowercase().as_str(), "none" | "off" | "no")
        {
            return Ok(Vec::new());
        }

        let mut targets = Vec::new();
        for part in trimmed.split(',') {
            let target = Self::parse(part)?;
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
        Ok(targets)
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Rs => "README.rs",
            Self::Html => "README.html",
            Self::Php => "README.php",
            Self::Python => "README.py",
            Self::TypeScript => "README.ts",
            Self::JavaScript => "README.js",
            Self::Go => "README.go",
            Self::Java => "README.java",
            Self::CSharp => "README.cs",
            Self::Kotlin => "README.kt",
            Self::Swift => "README.swift",
            Self::Ruby => "README.rb",
            Self::Json => "README.json",
        }
    }

    pub fn checkbox_label(&self) -> &'static str {
        match self {
            Self::Rs => "README.rs / Rust",
            Self::Html => "README.html / HTML5 + CSS3 + TypeScript",
            Self::Php => "README.php / PHP",
            Self::Python => "README.py / Python",
            Self::TypeScript => "README.ts / TypeScript",
            Self::JavaScript => "README.js / JavaScript",
            Self::Go => "README.go / Go",
            Self::Java => "README.java / Java",
            Self::CSharp => "README.cs / C#",
            Self::Kotlin => "README.kt / Kotlin",
            Self::Swift => "README.swift / Swift",
            Self::Ruby => "README.rb / Ruby",
            Self::Json => "README.json / JSON metadata",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Rs => "Rust projects can include README_HTML as a const string.",
            Self::Html => {
                "Responsive web page for smartphone, tablet, PC, WQHD, 4K, 8K, and 16K layouts."
            }
            Self::Php => {
                "PHP sites can echo the generated safe HTML without rewriting README content."
            }
            Self::Python => {
                "Python tools can import README_HTML for FastAPI, scripts, or documentation checks."
            }
            Self::TypeScript => "TypeScript apps can import strongly named README constants.",
            Self::JavaScript => {
                "JavaScript apps can import README constants for browser or Node usage."
            }
            Self::Go => "Go services can embed the generated README HTML.",
            Self::Java => "Java apps can include the generated README HTML as a class constant.",
            Self::CSharp => ".NET apps can include the generated README HTML as a static constant.",
            Self::Kotlin => {
                "Kotlin apps can include the generated README HTML as an object constant."
            }
            Self::Swift => "Swift apps can include the generated README HTML as a static constant.",
            Self::Ruby => "Ruby apps can include README_HTML for Rails or scripts.",
            Self::Json => "Machine-readable metadata for aruaru-ai, CI, and external tools.",
        }
    }

    pub fn all_menu_targets() -> Vec<Self> {
        vec![
            Self::Rs,
            Self::Html,
            Self::Php,
            Self::Python,
            Self::TypeScript,
            Self::JavaScript,
            Self::Go,
            Self::Java,
            Self::CSharp,
            Self::Kotlin,
            Self::Swift,
            Self::Ruby,
            Self::Json,
        ]
    }
}

pub fn build_readme_output_menu_plan(selected: &[ReadmeOutputTarget]) -> ReadmeOutputMenuPlan {
    let menu_items = ReadmeOutputTarget::all_menu_targets()
        .into_iter()
        .map(|target| ReadmeOutputMenuItem {
            target,
            checkbox_label: target.checkbox_label().to_string(),
            file_name: target.file_name().to_string(),
            description: target.description().to_string(),
            selected_by_default: selected.contains(&target),
        })
        .collect::<Vec<_>>();

    ReadmeOutputMenuPlan {
        fixed_output: "README.md is always created and kept as the canonical source.".to_string(),
        selected_outputs: selected.iter().map(|target| target.file_name().to_string()).collect(),
        menu_items,
        safety_rules: vec![
            "No checkbox selected means README.md only.".to_string(),
            "One or many checkbox targets may be selected.".to_string(),
            "Generated README.* files must never contain API keys, secrets, .env values, or SSH keys.".to_string(),
            "README.md remains the canonical source; generated files must be overwritten from README.md.".to_string(),
            "README.html must remain sanitized and responsive.".to_string(),
        ],
    }
}

pub fn quality_gate_smoke_check() -> bool {
    let selected = [
        ReadmeOutputTarget::Rs,
        ReadmeOutputTarget::Html,
        ReadmeOutputTarget::Php,
    ];
    let plan = build_readme_output_menu_plan(&selected);
    let parsed = ReadmeOutputTarget::parse_csv("rs,html,php,python").unwrap_or_default();
    plan.fixed_output.contains("README.md")
        && plan.selected_outputs.contains(&"README.rs".to_string())
        && parsed.contains(&ReadmeOutputTarget::Python)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_selection_is_readme_md_only() {
        let plan = build_readme_output_menu_plan(&[]);
        assert!(plan.selected_outputs.is_empty());
        assert!(plan.fixed_output.contains("README.md"));
    }

    #[test]
    fn multiple_targets_are_supported() {
        let selected =
            ReadmeOutputTarget::parse_csv("README.rs, README.html, php, TypeScript").unwrap();
        assert_eq!(
            selected,
            vec![
                ReadmeOutputTarget::Rs,
                ReadmeOutputTarget::Html,
                ReadmeOutputTarget::Php,
                ReadmeOutputTarget::TypeScript
            ]
        );
    }

    #[test]
    fn menu_includes_popular_languages() {
        let plan = build_readme_output_menu_plan(&[ReadmeOutputTarget::Rs]);
        let files = plan
            .menu_items
            .iter()
            .map(|item| item.file_name.as_str())
            .collect::<Vec<_>>();
        assert!(files.contains(&"README.rs"));
        assert!(files.contains(&"README.php"));
        assert!(files.contains(&"README.ts"));
        assert!(files.contains(&"README.py"));
        assert!(files.contains(&"README.go"));
    }
}
