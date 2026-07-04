//! Programming-language information planner for aruaru-ai.
//!
//! This module defines the data model used by the aruaru-ai development menu to
//! show language-by-language characteristics, benefits, drawbacks, maintenance
//! risks, and recommendation levels. The real crawler should refresh these facts
//! daily from official documentation, public ecosystem reports, and trusted Q&A /
//! repository trend sources, then store the summarized result with source metadata.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LanguageRecommendation {
    StrongDefault,
    Recommended,
    Situational,
    LegacyOrSpecialist,
    NotRecommendedByDefault,
}

impl LanguageRecommendation {
    pub fn label(&self) -> &'static str {
        match self {
            Self::StrongDefault => "strong default",
            Self::Recommended => "recommended",
            Self::Situational => "situational",
            Self::LegacyOrSpecialist => "legacy or specialist",
            Self::NotRecommendedByDefault => "not recommended by default",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LanguageSourceKind {
    OfficialDocumentation,
    EcosystemSurvey,
    PopularityIndex,
    RepositoryTrend,
    SecurityOrMaintenanceReport,
    InternalAruaruPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageInfoSource {
    pub title: String,
    pub url: String,
    pub kind: LanguageSourceKind,
    pub refresh_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgrammingLanguageInfo {
    pub language: String,
    pub recommended_for: Vec<String>,
    pub features: Vec<String>,
    pub merits: Vec<String>,
    pub demerits: Vec<String>,
    pub handoff_notes: Vec<String>,
    pub aruaru_recommendation: LanguageRecommendation,
    pub sources: Vec<LanguageInfoSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCrawlerPlan {
    pub title: String,
    pub schedule: String,
    pub source_priority: Vec<String>,
    pub update_steps: Vec<String>,
    pub safety_rules: Vec<String>,
    pub languages: Vec<ProgrammingLanguageInfo>,
}

fn official(title: &str, url: &str) -> LanguageInfoSource {
    LanguageInfoSource {
        title: title.to_string(),
        url: url.to_string(),
        kind: LanguageSourceKind::OfficialDocumentation,
        refresh_rule:
            "Check daily; treat official docs as the highest-priority source for language facts."
                .to_string(),
    }
}

fn ecosystem(title: &str, url: &str) -> LanguageInfoSource {
    LanguageInfoSource {
        title: title.to_string(),
        url: url.to_string(),
        kind: LanguageSourceKind::EcosystemSurvey,
        refresh_rule: "Check daily for new yearly or monthly ecosystem reports; store report date."
            .to_string(),
    }
}

fn internal_policy(title: &str) -> LanguageInfoSource {
    LanguageInfoSource {
        title: title.to_string(),
        url: "aruaru-ai://policy/programming-language-selection".to_string(),
        kind: LanguageSourceKind::InternalAruaruPolicy,
        refresh_rule:
            "Review when the project owner changes the aruaru-ai language selection policy."
                .to_string(),
    }
}

pub fn build_default_language_crawler_plan() -> LanguageCrawlerPlan {
    LanguageCrawlerPlan {
        title: "aruaru-ai programming language information crawler".to_string(),
        schedule: "Daily crawl and summarize once per day; do not overwrite user notes without version history.".to_string(),
        source_priority: vec![
            "Official language documentation and release notes".to_string(),
            "Major ecosystem reports such as GitHub Octoverse and Stack Overflow Developer Survey".to_string(),
            "Popularity indices such as TIOBE, clearly labeled as popularity rather than quality".to_string(),
            "Package registry, security advisory, and long-term-maintenance signals".to_string(),
            "aruaru-ai owner policy and local project experience".to_string(),
        ],
        update_steps: vec![
            "Fetch trusted sources with ETag / Last-Modified caching to avoid wasteful crawling.".to_string(),
            "Extract language facts, release status, popularity signals, maintenance warnings, and source dates.".to_string(),
            "Ask ChatGPT or the selected AI provider to summarize into feature / merit / demerit / handoff sections.".to_string(),
            "Store raw source metadata and AI summary separately so the UI can show evidence and detect stale text.".to_string(),
            "Run a contradiction check: official docs beat blogs; user policy beats generic popularity rankings.".to_string(),
            "Publish the new language info only after JSON schema validation and secret-scan pass.".to_string(),
        ],
        safety_rules: vec![
            "Do not copy long source text; summarize and cite source URLs / titles / retrieval dates.".to_string(),
            "Do not treat popularity as technical superiority.".to_string(),
            "Ruby is allowed as a legacy checkbox, but aruaru-ai should mark it not recommended by default unless the user explicitly selects it.".to_string(),
            "Do not crawl private repositories, paid docs, or user secrets.".to_string(),
            "Keep daily versions so a bad AI summary can be rolled back.".to_string(),
        ],
        languages: default_programming_language_infos(),
    }
}

pub fn default_programming_language_infos() -> Vec<ProgrammingLanguageInfo> {
    vec![
        ProgrammingLanguageInfo {
            language: "Rust".to_string(),
            recommended_for: vec!["aruaru core backend".to_string(), "Poem web server".to_string(), "high reliability tooling".to_string()],
            features: vec!["memory safety without a garbage collector".to_string(), "strong static type system".to_string(), "Cargo package and build workflow".to_string()],
            merits: vec!["low runtime overhead and strong correctness checks".to_string(), "excellent fit for long-running infrastructure".to_string(), "good for reducing runtime bugs before deployment".to_string()],
            demerits: vec!["learning curve is higher than scripting languages".to_string(), "compile-time errors can be difficult for beginners".to_string()],
            handoff_notes: vec!["Use fmt, clippy, tests, clear module boundaries, and small functions to make handoff easier.".to_string()],
            aruaru_recommendation: LanguageRecommendation::StrongDefault,
            sources: vec![official("Rust documentation", "https://www.rust-lang.org/learn"), ecosystem("GitHub Octoverse", "https://github.blog/news-insights/octoverse/")],
        },
        ProgrammingLanguageInfo {
            language: "TypeScript".to_string(),
            recommended_for: vec!["frontend".to_string(), "typed browser code".to_string(), "GraphQL client code".to_string()],
            features: vec!["JavaScript with static type annotations".to_string(), "works with modern browser and Node.js tooling".to_string()],
            merits: vec!["better maintainability than plain JavaScript for larger UI code".to_string(), "good ecosystem for web apps".to_string()],
            demerits: vec!["build tooling can become complex".to_string(), "types can be bypassed or become misleading if poorly maintained".to_string()],
            handoff_notes: vec!["Prefer strict mode, generated GraphQL types, and no plain JavaScript for aruaru default code.".to_string()],
            aruaru_recommendation: LanguageRecommendation::StrongDefault,
            sources: vec![official("TypeScript documentation", "https://www.typescriptlang.org/docs/"), ecosystem("GitHub Octoverse", "https://github.blog/news-insights/octoverse/")],
        },
        ProgrammingLanguageInfo {
            language: "Python".to_string(),
            recommended_for: vec!["AI tools".to_string(), "data processing".to_string(), "automation scripts".to_string()],
            features: vec!["readable dynamic language".to_string(), "large AI and data science ecosystem".to_string()],
            merits: vec!["fast prototyping".to_string(), "many libraries and learning resources".to_string()],
            demerits: vec!["runtime type errors are easier to miss without tests and type checking".to_string(), "deployment and dependency isolation need discipline".to_string()],
            handoff_notes: vec!["Use pyproject.toml, type hints, ruff/mypy/pytest, and small modules.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Recommended,
            sources: vec![official("Python documentation", "https://docs.python.org/3/"), ecosystem("TIOBE Index", "https://www.tiobe.com/tiobe-index/")],
        },
        ProgrammingLanguageInfo {
            language: "PHP".to_string(),
            recommended_for: vec!["legacy web hosting".to_string(), "WordPress-compatible bridges".to_string(), "small web apps".to_string()],
            features: vec!["server-side web language".to_string(), "widely available on shared hosting".to_string()],
            merits: vec!["very easy deployment on many rental servers".to_string(), "large CMS ecosystem".to_string()],
            demerits: vec!["legacy code quality varies widely".to_string(), "large projects need strict framework and coding standards".to_string()],
            handoff_notes: vec!["Use modern PHP, Composer, static analysis, and framework conventions if selected.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("PHP manual", "https://www.php.net/manual/en/"), ecosystem("TIOBE Index", "https://www.tiobe.com/tiobe-index/")],
        },
        ProgrammingLanguageInfo {
            language: "Go".to_string(),
            recommended_for: vec!["simple services".to_string(), "CLI tools".to_string(), "cloud infrastructure".to_string()],
            features: vec!["compiled language with simple syntax".to_string(), "built-in concurrency primitives".to_string()],
            merits: vec!["easy deployment as a single binary".to_string(), "good readability for teams".to_string()],
            demerits: vec!["less expressive type system than Rust for some domains".to_string(), "error handling can be verbose".to_string()],
            handoff_notes: vec!["Use gofmt, clear packages, and explicit interface boundaries.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Recommended,
            sources: vec![official("Go documentation", "https://go.dev/doc/"), ecosystem("Stack Overflow Developer Survey", "https://survey.stackoverflow.co/")],
        },
        ProgrammingLanguageInfo {
            language: "Java".to_string(),
            recommended_for: vec!["enterprise systems".to_string(), "Spring Boot".to_string(), "large teams".to_string()],
            features: vec!["mature JVM ecosystem".to_string(), "strong tooling and long-term support culture".to_string()],
            merits: vec!["many enterprise libraries".to_string(), "good maintainability when conventions are enforced".to_string()],
            demerits: vec!["can be verbose".to_string(), "framework-heavy projects may become complex".to_string()],
            handoff_notes: vec!["Use standard architecture, dependency injection carefully, and automated tests.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("Java documentation", "https://docs.oracle.com/en/java/"), ecosystem("Stack Overflow Developer Survey", "https://survey.stackoverflow.co/")],
        },
        ProgrammingLanguageInfo {
            language: "C#".to_string(),
            recommended_for: vec!["Windows apps".to_string(), ".NET web systems".to_string(), "enterprise tooling".to_string()],
            features: vec!["modern statically typed .NET language".to_string(), "strong IDE and runtime ecosystem".to_string()],
            merits: vec!["good productivity on Windows".to_string(), "mature web and desktop options".to_string()],
            demerits: vec!["ecosystem choices can lock projects into .NET conventions".to_string(), "cross-platform packaging needs planning".to_string()],
            handoff_notes: vec!["Use analyzers, nullable reference types, and clear solution structure.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("C# documentation", "https://learn.microsoft.com/dotnet/csharp/"), ecosystem("Stack Overflow Developer Survey", "https://survey.stackoverflow.co/")],
        },
        ProgrammingLanguageInfo {
            language: "Kotlin".to_string(),
            recommended_for: vec!["Android".to_string(), "Ktor".to_string(), "JVM apps".to_string()],
            features: vec!["concise JVM language".to_string(), "null-safety features".to_string()],
            merits: vec!["good Android fit".to_string(), "less boilerplate than Java".to_string()],
            demerits: vec!["build tooling can be heavy".to_string(), "team experience varies outside Android".to_string()],
            handoff_notes: vec!["Use Gradle conventions and avoid over-clever DSL magic.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("Kotlin documentation", "https://kotlinlang.org/docs/home.html")],
        },
        ProgrammingLanguageInfo {
            language: "Swift".to_string(),
            recommended_for: vec!["iOS".to_string(), "macOS".to_string(), "Apple ecosystem apps".to_string()],
            features: vec!["Apple-first compiled language".to_string(), "modern syntax and safety features".to_string()],
            merits: vec!["best fit for native Apple apps".to_string(), "strong platform integration".to_string()],
            demerits: vec!["less natural for non-Apple server and VPS workflows".to_string(), "Apple toolchain knowledge is needed".to_string()],
            handoff_notes: vec!["Use clear SwiftUI/UIKit boundaries and platform-specific docs.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("Swift documentation", "https://www.swift.org/documentation/")],
        },
        ProgrammingLanguageInfo {
            language: "Ruby".to_string(),
            recommended_for: vec!["existing Ruby on Rails systems".to_string(), "maintenance of legacy Ruby code".to_string()],
            features: vec!["Japanese creator and strong Japanese community history".to_string(), "expressive dynamic syntax".to_string(), "Rails ecosystem for web apps".to_string()],
            merits: vec!["Japanese learning resources are comparatively accessible".to_string(), "Rails can be productive when the team already has Rails expertise".to_string()],
            demerits: vec!["dynamic and expressive code can become hard to hand over when style is inconsistent".to_string(), "long-term maintainability depends heavily on the original developer's discipline".to_string(), "aruaru-ai should not choose it by default for new multi-developer projects".to_string()],
            handoff_notes: vec!["User policy: do not recommend Ruby by default; allow it only for explicit Ruby/Rails maintenance or when the user has a strong reason.".to_string()],
            aruaru_recommendation: LanguageRecommendation::NotRecommendedByDefault,
            sources: vec![official("Ruby documentation", "https://www.ruby-lang.org/en/documentation/"), official("Ruby on Rails Guides", "https://guides.rubyonrails.org/"), internal_policy("aruaru-ai owner policy: Ruby is not recommended by default")],
        },
        ProgrammingLanguageInfo {
            language: "C++".to_string(),
            recommended_for: vec!["performance-critical native code".to_string(), "game engines".to_string(), "hardware-adjacent systems".to_string()],
            features: vec!["high-performance compiled language".to_string(), "large existing ecosystem".to_string()],
            merits: vec!["very high performance and control".to_string(), "many libraries for native systems".to_string()],
            demerits: vec!["memory safety and build complexity are major risks".to_string(), "handoff requires strict standards".to_string()],
            handoff_notes: vec!["Use modern C++, sanitizers, static analysis, and narrow module boundaries.".to_string()],
            aruaru_recommendation: LanguageRecommendation::LegacyOrSpecialist,
            sources: vec![official("C++ reference", "https://en.cppreference.com/w/"), ecosystem("TIOBE Index", "https://www.tiobe.com/tiobe-index/")],
        },
        ProgrammingLanguageInfo {
            language: "C".to_string(),
            recommended_for: vec!["embedded systems".to_string(), "OS-level integration".to_string(), "very low-level libraries".to_string()],
            features: vec!["small low-level compiled language".to_string(), "close hardware control".to_string()],
            merits: vec!["universal systems language".to_string(), "minimal runtime assumptions".to_string()],
            demerits: vec!["manual memory management creates serious bug risk".to_string(), "not ideal for ordinary web/app development".to_string()],
            handoff_notes: vec!["Use only where the low-level requirement is real; otherwise prefer Rust for aruaru new work.".to_string()],
            aruaru_recommendation: LanguageRecommendation::LegacyOrSpecialist,
            sources: vec![ecosystem("TIOBE Index", "https://www.tiobe.com/tiobe-index/")],
        },
        ProgrammingLanguageInfo {
            language: "Zig".to_string(),
            recommended_for: vec!["experimental systems tooling".to_string(), "C interop experiments".to_string()],
            features: vec!["systems language with explicit control".to_string(), "focus on simplicity and compile-time features".to_string()],
            merits: vec!["promising for systems experiments".to_string(), "good C interop direction".to_string()],
            demerits: vec!["ecosystem is younger than Rust, Go, Java, or Python".to_string(), "not yet the safest default for business apps".to_string()],
            handoff_notes: vec!["Treat as experimental unless the team has specific Zig expertise.".to_string()],
            aruaru_recommendation: LanguageRecommendation::Situational,
            sources: vec![official("Zig documentation", "https://ziglang.org/documentation/master/")],
        },
    ]
}

pub fn language_info_markdown() -> String {
    let plan = build_default_language_crawler_plan();
    let mut out = String::new();
    out.push_str("# aruaru-ai Programming Language Information\n\n");
    out.push_str(&format!("Schedule: {}\n\n", plan.schedule));
    out.push_str("## Source priority\n");
    for item in &plan.source_priority {
        out.push_str(&format!("- {}\n", item));
    }
    out.push_str("\n## Languages\n");
    for lang in &plan.languages {
        out.push_str(&format!("\n### {}\n", lang.language));
        out.push_str(&format!(
            "Recommendation: {}\n",
            lang.aruaru_recommendation.label()
        ));
        out.push_str("\nFeatures:\n");
        for item in &lang.features {
            out.push_str(&format!("- {}\n", item));
        }
        out.push_str("Merits:\n");
        for item in &lang.merits {
            out.push_str(&format!("- {}\n", item));
        }
        out.push_str("Demerits:\n");
        for item in &lang.demerits {
            out.push_str(&format!("- {}\n", item));
        }
    }
    out
}

pub fn quality_gate_smoke_check() -> bool {
    let plan = build_default_language_crawler_plan();
    let ruby = plan.languages.iter().find(|item| item.language == "Ruby");
    let rust = plan.languages.iter().find(|item| item.language == "Rust");

    ruby.is_some_and(|item| {
        item.aruaru_recommendation == LanguageRecommendation::NotRecommendedByDefault
    }) && rust
        .is_some_and(|item| item.aruaru_recommendation == LanguageRecommendation::StrongDefault)
        && plan.schedule.contains("Daily")
        && plan.safety_rules.iter().any(|rule| rule.contains("Ruby"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_crawler_plan_contains_source_priority() {
        let plan = build_default_language_crawler_plan();
        assert!(plan.schedule.contains("Daily"));
        assert!(plan
            .source_priority
            .iter()
            .any(|item| item.contains("Official")));
        assert!(plan
            .update_steps
            .iter()
            .any(|item| item.contains("ChatGPT")));
    }

    #[test]
    fn ruby_is_not_recommended_by_default() {
        let plan = build_default_language_crawler_plan();
        let ruby = plan
            .languages
            .iter()
            .find(|item| item.language == "Ruby")
            .unwrap();
        assert_eq!(
            ruby.aruaru_recommendation,
            LanguageRecommendation::NotRecommendedByDefault
        );
        assert!(ruby
            .handoff_notes
            .iter()
            .any(|note| note.contains("do not recommend Ruby by default")));
    }

    #[test]
    fn rust_and_typescript_are_strong_defaults() {
        let plan = build_default_language_crawler_plan();
        let strong = plan
            .languages
            .iter()
            .filter(|item| item.aruaru_recommendation == LanguageRecommendation::StrongDefault)
            .map(|item| item.language.as_str())
            .collect::<Vec<_>>();
        assert!(strong.contains(&"Rust"));
        assert!(strong.contains(&"TypeScript"));
    }

    #[test]
    fn markdown_contains_merits_and_demerits() {
        let markdown = language_info_markdown();
        assert!(markdown.contains("Merits:"));
        assert!(markdown.contains("Demerits:"));
        assert!(markdown.contains("Ruby"));
    }
}
