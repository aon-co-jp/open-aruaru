//! aruaru-ai development menu planner.
//!
//! aruaru-ai must let the user select none, one, or many choices from each
//! development category. README.md stays fixed, while frontend, programming
//! language, framework, database, AI provider, packaging, and operations choices
//! are optional checkbox groups.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DevelopmentCategory {
    Frontend,
    ProgrammingLanguage,
    BackendFramework,
    FrontendFramework,
    Database,
    ApiStyle,
    AiProvider,
    LocalLlmRuntime,
    DevOps,
    PackageTarget,
    QualityGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DevelopmentMenuTarget {
    // Frontend / UI
    Html5Css3TypeScript,
    ResponsiveWeb,
    Pwa,
    DesktopWebView,
    AndroidWeb,
    IosWeb,

    // Programming languages
    Rust,
    TypeScript,
    Python,
    Php,
    Go,
    Java,
    CSharp,
    Kotlin,
    Swift,
    Ruby,
    Cpp,
    C,
    Zig,

    // Backend frameworks
    Poem,
    Axum,
    ActixWeb,
    FastApi,
    Django,
    Laravel,
    Symfony,
    Gin,
    SpringBoot,
    AspNetCore,
    Ktor,
    Rails,

    // Frontend frameworks
    VanillaTypeScript,
    React,
    Vue,
    Svelte,
    Solid,
    Angular,
    Astro,
    NextJs,
    Nuxt,

    // Databases / storage
    PostgreSql,
    CockroachDb,
    SQLite,
    MySql,
    MariaDb,
    MongoDb,
    Redis,
    ClickHouse,
    DuckDb,
    Elasticsearch,
    OpenSearch,
    S3CompatibleStorage,

    // API / schema
    GraphQl,
    WunderGraphCosmo,
    Grpc,
    WebSocket,
    Sse,

    // AI providers / runtimes
    OpenAiChatGpt,
    AnthropicClaudeOpus,
    GoogleGemini,
    DeepSeekApi,
    LocalAruaruLlm,
    OpenCuda,
    Ollama,
    LmStudio,
    Gguf,

    // DevOps / deployment
    Docker,
    Podman,
    Kubernetes,
    Systemd,
    Nginx,
    ApacheHttpd,
    Caddy,
    LetsEncrypt,
    ConohaVps,
    GitHubActions,
    RedmineReport,

    // Package targets
    WindowsZip,
    LinuxTarGz,
    MacOsAppBundle,
    AndroidPackage,
    IosPackage,
    WebDeployFolder,

    // Quality gates
    CargoFmt,
    CargoCheck,
    CargoTest,
    CargoClippyDWarnings,
    PowerShellBugCheck,
    SecretScan,
    ReadmeGenerationCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevelopmentMenuItem {
    pub target: DevelopmentMenuTarget,
    pub category: DevelopmentCategory,
    pub checkbox_label: String,
    pub description: String,
    pub selected_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevelopmentMenuPlan {
    pub menu_title: String,
    pub selection_rule: String,
    pub fixed_rules: Vec<String>,
    pub selected_labels: Vec<String>,
    pub menu_items: Vec<DevelopmentMenuItem>,
    pub safety_rules: Vec<String>,
}

impl DevelopmentCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Frontend => "Frontend / UI",
            Self::ProgrammingLanguage => "Programming language",
            Self::BackendFramework => "Backend framework",
            Self::FrontendFramework => "Frontend framework",
            Self::Database => "Database / storage",
            Self::ApiStyle => "API / schema",
            Self::AiProvider => "AI provider",
            Self::LocalLlmRuntime => "Local LLM runtime",
            Self::DevOps => "DevOps / deployment",
            Self::PackageTarget => "Package target",
            Self::QualityGate => "Quality gate",
        }
    }
}

impl DevelopmentMenuTarget {
    pub fn parse(value: &str) -> Result<Self> {
        let lowered = value.trim().to_ascii_lowercase();
        let normalized = lowered
            .replace("readme", "")
            .replace([' ', '-', '_', '.', '/', '+'], "");

        match normalized.as_str() {
            "html5css3typescript" | "htmlcssjs" | "htmlcss_ts" => Ok(Self::Html5Css3TypeScript),
            "responsiveweb" | "responsive" => Ok(Self::ResponsiveWeb),
            "pwa" => Ok(Self::Pwa),
            "desktopwebview" | "webview" => Ok(Self::DesktopWebView),
            "androidweb" | "android" => Ok(Self::AndroidWeb),
            "iosweb" | "iphone" | "ios" => Ok(Self::IosWeb),
            "rust" | "rs" => Ok(Self::Rust),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "python" | "py" => Ok(Self::Python),
            "php" => Ok(Self::Php),
            "go" | "golang" => Ok(Self::Go),
            "java" => Ok(Self::Java),
            "csharp" | "cs" | "c#" => Ok(Self::CSharp),
            "kotlin" | "kt" => Ok(Self::Kotlin),
            "swift" => Ok(Self::Swift),
            "ruby" | "rb" => Ok(Self::Ruby),
            "cpp" | "c++" => Ok(Self::Cpp),
            "c" => Ok(Self::C),
            "zig" => Ok(Self::Zig),
            "poem" => Ok(Self::Poem),
            "axum" => Ok(Self::Axum),
            "actixweb" | "actix" => Ok(Self::ActixWeb),
            "fastapi" => Ok(Self::FastApi),
            "django" => Ok(Self::Django),
            "laravel" => Ok(Self::Laravel),
            "symfony" => Ok(Self::Symfony),
            "gin" | "gingonic" => Ok(Self::Gin),
            "springboot" | "spring" => Ok(Self::SpringBoot),
            "aspnetcore" | "dotnet" | "aspnet" => Ok(Self::AspNetCore),
            "ktor" => Ok(Self::Ktor),
            "rails" | "rubyonrails" => Ok(Self::Rails),
            "vanillatypescript" | "vanillats" => Ok(Self::VanillaTypeScript),
            "react" => Ok(Self::React),
            "vue" | "vuejs" => Ok(Self::Vue),
            "svelte" => Ok(Self::Svelte),
            "solid" | "solidjs" => Ok(Self::Solid),
            "angular" => Ok(Self::Angular),
            "astro" => Ok(Self::Astro),
            "nextjs" | "next" => Ok(Self::NextJs),
            "nuxt" | "nuxtjs" => Ok(Self::Nuxt),
            "postgresql" | "postgres" | "pgsql" => Ok(Self::PostgreSql),
            "cockroachdb" | "cockroach" => Ok(Self::CockroachDb),
            "sqlite" => Ok(Self::SQLite),
            "mysql" => Ok(Self::MySql),
            "mariadb" | "maria" => Ok(Self::MariaDb),
            "mongodb" | "mongo" => Ok(Self::MongoDb),
            "redis" => Ok(Self::Redis),
            "clickhouse" => Ok(Self::ClickHouse),
            "duckdb" => Ok(Self::DuckDb),
            "elasticsearch" => Ok(Self::Elasticsearch),
            "opensearch" => Ok(Self::OpenSearch),
            "s3" | "s3compatiblestorage" => Ok(Self::S3CompatibleStorage),
            "graphql" => Ok(Self::GraphQl),
            "wundergraphcosmo" | "cosmo" | "wundercosmo" => Ok(Self::WunderGraphCosmo),
            "grpc" => Ok(Self::Grpc),
            "websocket" | "websockets" => Ok(Self::WebSocket),
            "sse" | "serversentevents" => Ok(Self::Sse),
            "openai" | "chatgpt" | "openaichatgpt" => Ok(Self::OpenAiChatGpt),
            "anthropic" | "claude" | "opus" | "anthropicclaudeopus" => {
                Ok(Self::AnthropicClaudeOpus)
            }
            "gemini" | "googlegemini" => Ok(Self::GoogleGemini),
            "deepseek" | "deepseekapi" => Ok(Self::DeepSeekApi),
            "aruarullm" | "localaruarullm" => Ok(Self::LocalAruaruLlm),
            "opencuda" => Ok(Self::OpenCuda),
            "ollama" => Ok(Self::Ollama),
            "lmstudio" => Ok(Self::LmStudio),
            "gguf" => Ok(Self::Gguf),
            "docker" => Ok(Self::Docker),
            "podman" => Ok(Self::Podman),
            "kubernetes" | "k8s" => Ok(Self::Kubernetes),
            "systemd" => Ok(Self::Systemd),
            "nginx" => Ok(Self::Nginx),
            "apache" | "apachehttpd" | "httpd" => Ok(Self::ApacheHttpd),
            "caddy" => Ok(Self::Caddy),
            "letsencrypt" | "certbot" => Ok(Self::LetsEncrypt),
            "conohavps" | "conoha" => Ok(Self::ConohaVps),
            "githubactions" | "githubci" => Ok(Self::GitHubActions),
            "redminereport" | "redmine" => Ok(Self::RedmineReport),
            "windowszip" | "windows" => Ok(Self::WindowsZip),
            "linuxtargz" | "linux" => Ok(Self::LinuxTarGz),
            "macosappbundle" | "macos" | "mac" => Ok(Self::MacOsAppBundle),
            "androidpackage" | "apk" | "aab" => Ok(Self::AndroidPackage),
            "iospackage" | "ipa" => Ok(Self::IosPackage),
            "webdeployfolder" | "webdeploy" => Ok(Self::WebDeployFolder),
            "cargofmt" | "fmt" => Ok(Self::CargoFmt),
            "cargocheck" => Ok(Self::CargoCheck),
            "cargotest" | "test" => Ok(Self::CargoTest),
            "cargoclippy" | "clippy" | "clippydwarnings" => Ok(Self::CargoClippyDWarnings),
            "powershellbugcheck" | "psbugcheck" => Ok(Self::PowerShellBugCheck),
            "secretscan" => Ok(Self::SecretScan),
            "readmegenerationcheck" | "readmecheck" => Ok(Self::ReadmeGenerationCheck),
            "" | "none" | "off" | "no" => Err(anyhow!("empty development menu target")),
            other => Err(anyhow!("unknown development menu target: {other}")),
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

    pub fn category(&self) -> DevelopmentCategory {
        match self {
            Self::Html5Css3TypeScript
            | Self::ResponsiveWeb
            | Self::Pwa
            | Self::DesktopWebView
            | Self::AndroidWeb
            | Self::IosWeb => DevelopmentCategory::Frontend,
            Self::Rust
            | Self::TypeScript
            | Self::Python
            | Self::Php
            | Self::Go
            | Self::Java
            | Self::CSharp
            | Self::Kotlin
            | Self::Swift
            | Self::Ruby
            | Self::Cpp
            | Self::C
            | Self::Zig => DevelopmentCategory::ProgrammingLanguage,
            Self::Poem
            | Self::Axum
            | Self::ActixWeb
            | Self::FastApi
            | Self::Django
            | Self::Laravel
            | Self::Symfony
            | Self::Gin
            | Self::SpringBoot
            | Self::AspNetCore
            | Self::Ktor
            | Self::Rails => DevelopmentCategory::BackendFramework,
            Self::VanillaTypeScript
            | Self::React
            | Self::Vue
            | Self::Svelte
            | Self::Solid
            | Self::Angular
            | Self::Astro
            | Self::NextJs
            | Self::Nuxt => DevelopmentCategory::FrontendFramework,
            Self::PostgreSql
            | Self::CockroachDb
            | Self::SQLite
            | Self::MySql
            | Self::MariaDb
            | Self::MongoDb
            | Self::Redis
            | Self::ClickHouse
            | Self::DuckDb
            | Self::Elasticsearch
            | Self::OpenSearch
            | Self::S3CompatibleStorage => DevelopmentCategory::Database,
            Self::GraphQl | Self::WunderGraphCosmo | Self::Grpc | Self::WebSocket | Self::Sse => {
                DevelopmentCategory::ApiStyle
            }
            Self::OpenAiChatGpt
            | Self::AnthropicClaudeOpus
            | Self::GoogleGemini
            | Self::DeepSeekApi => DevelopmentCategory::AiProvider,
            Self::LocalAruaruLlm | Self::OpenCuda | Self::Ollama | Self::LmStudio | Self::Gguf => {
                DevelopmentCategory::LocalLlmRuntime
            }
            Self::Docker
            | Self::Podman
            | Self::Kubernetes
            | Self::Systemd
            | Self::Nginx
            | Self::ApacheHttpd
            | Self::Caddy
            | Self::LetsEncrypt
            | Self::ConohaVps
            | Self::GitHubActions
            | Self::RedmineReport => DevelopmentCategory::DevOps,
            Self::WindowsZip
            | Self::LinuxTarGz
            | Self::MacOsAppBundle
            | Self::AndroidPackage
            | Self::IosPackage
            | Self::WebDeployFolder => DevelopmentCategory::PackageTarget,
            Self::CargoFmt
            | Self::CargoCheck
            | Self::CargoTest
            | Self::CargoClippyDWarnings
            | Self::PowerShellBugCheck
            | Self::SecretScan
            | Self::ReadmeGenerationCheck => DevelopmentCategory::QualityGate,
        }
    }

    pub fn checkbox_label(&self) -> &'static str {
        match self {
            Self::Html5Css3TypeScript => "HTML5 + CSS3 + TypeScript",
            Self::ResponsiveWeb => "Responsive Web / smartphone-tablet-PC-4K",
            Self::Pwa => "PWA",
            Self::DesktopWebView => "Desktop WebView shell",
            Self::AndroidWeb => "Android web package",
            Self::IosWeb => "iPhone / iPad web package",
            Self::Rust => "Rust",
            Self::TypeScript => "TypeScript",
            Self::Python => "Python",
            Self::Php => "PHP",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::CSharp => "C# / .NET",
            Self::Kotlin => "Kotlin",
            Self::Swift => "Swift",
            Self::Ruby => "Ruby",
            Self::Cpp => "C++",
            Self::C => "C",
            Self::Zig => "Zig",
            Self::Poem => "Poem / Rust web framework",
            Self::Axum => "Axum / Rust web framework",
            Self::ActixWeb => "Actix Web / Rust web framework",
            Self::FastApi => "FastAPI / Python",
            Self::Django => "Django / Python",
            Self::Laravel => "Laravel / PHP",
            Self::Symfony => "Symfony / PHP",
            Self::Gin => "Gin / Go",
            Self::SpringBoot => "Spring Boot / Java",
            Self::AspNetCore => "ASP.NET Core / C#",
            Self::Ktor => "Ktor / Kotlin",
            Self::Rails => "Ruby on Rails",
            Self::VanillaTypeScript => "Vanilla TypeScript",
            Self::React => "React",
            Self::Vue => "Vue",
            Self::Svelte => "Svelte",
            Self::Solid => "Solid",
            Self::Angular => "Angular",
            Self::Astro => "Astro",
            Self::NextJs => "Next.js",
            Self::Nuxt => "Nuxt",
            Self::PostgreSql => "PostgreSQL",
            Self::CockroachDb => "CockroachDB",
            Self::SQLite => "SQLite",
            Self::MySql => "MySQL",
            Self::MariaDb => "MariaDB",
            Self::MongoDb => "MongoDB",
            Self::Redis => "Redis",
            Self::ClickHouse => "ClickHouse",
            Self::DuckDb => "DuckDB",
            Self::Elasticsearch => "Elasticsearch",
            Self::OpenSearch => "OpenSearch",
            Self::S3CompatibleStorage => "S3-compatible object storage",
            Self::GraphQl => "GraphQL",
            Self::WunderGraphCosmo => "WunderGraph Cosmo",
            Self::Grpc => "gRPC",
            Self::WebSocket => "WebSocket",
            Self::Sse => "SSE / Server-Sent Events",
            Self::OpenAiChatGpt => "ChatGPT / OpenAI API",
            Self::AnthropicClaudeOpus => "Claude / Opus / Anthropic API",
            Self::GoogleGemini => "Gemini / Google API",
            Self::DeepSeekApi => "DeepSeek API",
            Self::LocalAruaruLlm => "aruaru-llm local runtime",
            Self::OpenCuda => "OpenCUDA project runtime",
            Self::Ollama => "Ollama",
            Self::LmStudio => "LM Studio",
            Self::Gguf => "GGUF local model",
            Self::Docker => "Docker",
            Self::Podman => "Podman",
            Self::Kubernetes => "Kubernetes",
            Self::Systemd => "systemd",
            Self::Nginx => "Nginx",
            Self::ApacheHttpd => "Apache httpd",
            Self::Caddy => "Caddy",
            Self::LetsEncrypt => "Let's Encrypt / Certbot",
            Self::ConohaVps => "ConoHa VPS",
            Self::GitHubActions => "GitHub Actions",
            Self::RedmineReport => "Redmine report export",
            Self::WindowsZip => "Windows ZIP package",
            Self::LinuxTarGz => "Linux tar.gz package",
            Self::MacOsAppBundle => "macOS app bundle",
            Self::AndroidPackage => "Android APK/AAB package",
            Self::IosPackage => "iOS package",
            Self::WebDeployFolder => "Web deploy folder",
            Self::CargoFmt => "cargo fmt",
            Self::CargoCheck => "cargo check",
            Self::CargoTest => "cargo test",
            Self::CargoClippyDWarnings => "cargo clippy -- -D warnings",
            Self::PowerShellBugCheck => "PowerShell bug check script",
            Self::SecretScan => "Secret / API key scan",
            Self::ReadmeGenerationCheck => "README generation check",
        }
    }

    pub fn description(&self) -> &'static str {
        match self.category() {
            DevelopmentCategory::Frontend => "Optional UI/runtime target for apps and websites.",
            DevelopmentCategory::ProgrammingLanguage => "Selectable implementation language; multiple languages may be chosen for plugins or bridges.",
            DevelopmentCategory::BackendFramework => "Backend framework choice. aruaru's default remains Rust + Poem unless the user selects otherwise.",
            DevelopmentCategory::FrontendFramework => "Frontend framework choice. TypeScript is preferred over plain JavaScript for generated code.",
            DevelopmentCategory::Database => "Database or storage engine. Multiple selections can be used for migration, backup, or compatibility.",
            DevelopmentCategory::ApiStyle => "API/schema transport. Internal aruaru API should prefer GraphQL and WunderGraph Cosmo.",
            DevelopmentCategory::AiProvider => "External AI provider API. API keys must be imported only from user-owned keys and never logged.",
            DevelopmentCategory::LocalLlmRuntime => "Local LLM runtime or local GPU runtime option. The app should validate existing local paths before downloads.",
            DevelopmentCategory::DevOps => "Deployment, HTTPS, VPS, or CI option.",
            DevelopmentCategory::PackageTarget => "Distribution target. Multiple targets may be generated from one project plan.",
            DevelopmentCategory::QualityGate => "Quality gate to reduce BUG returns and prevent secret leakage.",
        }
    }

    pub fn all_menu_targets() -> Vec<Self> {
        vec![
            Self::Html5Css3TypeScript,
            Self::ResponsiveWeb,
            Self::Pwa,
            Self::DesktopWebView,
            Self::AndroidWeb,
            Self::IosWeb,
            Self::Rust,
            Self::TypeScript,
            Self::Python,
            Self::Php,
            Self::Go,
            Self::Java,
            Self::CSharp,
            Self::Kotlin,
            Self::Swift,
            Self::Ruby,
            Self::Cpp,
            Self::C,
            Self::Zig,
            Self::Poem,
            Self::Axum,
            Self::ActixWeb,
            Self::FastApi,
            Self::Django,
            Self::Laravel,
            Self::Symfony,
            Self::Gin,
            Self::SpringBoot,
            Self::AspNetCore,
            Self::Ktor,
            Self::Rails,
            Self::VanillaTypeScript,
            Self::React,
            Self::Vue,
            Self::Svelte,
            Self::Solid,
            Self::Angular,
            Self::Astro,
            Self::NextJs,
            Self::Nuxt,
            Self::PostgreSql,
            Self::CockroachDb,
            Self::SQLite,
            Self::MySql,
            Self::MariaDb,
            Self::MongoDb,
            Self::Redis,
            Self::ClickHouse,
            Self::DuckDb,
            Self::Elasticsearch,
            Self::OpenSearch,
            Self::S3CompatibleStorage,
            Self::GraphQl,
            Self::WunderGraphCosmo,
            Self::Grpc,
            Self::WebSocket,
            Self::Sse,
            Self::OpenAiChatGpt,
            Self::AnthropicClaudeOpus,
            Self::GoogleGemini,
            Self::DeepSeekApi,
            Self::LocalAruaruLlm,
            Self::OpenCuda,
            Self::Ollama,
            Self::LmStudio,
            Self::Gguf,
            Self::Docker,
            Self::Podman,
            Self::Kubernetes,
            Self::Systemd,
            Self::Nginx,
            Self::ApacheHttpd,
            Self::Caddy,
            Self::LetsEncrypt,
            Self::ConohaVps,
            Self::GitHubActions,
            Self::RedmineReport,
            Self::WindowsZip,
            Self::LinuxTarGz,
            Self::MacOsAppBundle,
            Self::AndroidPackage,
            Self::IosPackage,
            Self::WebDeployFolder,
            Self::CargoFmt,
            Self::CargoCheck,
            Self::CargoTest,
            Self::CargoClippyDWarnings,
            Self::PowerShellBugCheck,
            Self::SecretScan,
            Self::ReadmeGenerationCheck,
        ]
    }
}

pub fn build_development_menu_plan(selected: &[DevelopmentMenuTarget]) -> DevelopmentMenuPlan {
    let menu_items = DevelopmentMenuTarget::all_menu_targets()
        .into_iter()
        .map(|target| DevelopmentMenuItem {
            target,
            category: target.category(),
            checkbox_label: target.checkbox_label().to_string(),
            description: target.description().to_string(),
            selected_by_default: selected.contains(&target),
        })
        .collect::<Vec<_>>();

    DevelopmentMenuPlan {
        menu_title: "aruaru-ai development menu".to_string(),
        selection_rule: "Every category supports none, one, or many checkbox selections.".to_string(),
        fixed_rules: vec![
            "README.md is always generated as the canonical project document.".to_string(),
            "A non-standard desktop shell is not part of the aruaru default stack.".to_string(),
            "Internal aruaru APIs should prefer GraphQL and WunderGraph Cosmo.".to_string(),
            "TypeScript is preferred over plain JavaScript for generated frontend code.".to_string(),
            "Rust + Poem remains the default backend choice for the aruaru core.".to_string(),
        ],
        selected_labels: selected.iter().map(|target| target.checkbox_label().to_string()).collect(),
        menu_items,
        safety_rules: vec![
            "Never include API keys, .env values, SSH keys, tokens, or private certificates in generated menus or reports.".to_string(),
            "When multiple databases are selected, generate explicit migration and backup rules instead of silently duplicating writes.".to_string(),
            "When multiple AI providers are selected, use cost limits, redaction, and user approval before sending project files.".to_string(),
            "When local LLM runtimes are selected, validate existing paths and model licenses before any download.".to_string(),
            "Quality gates should run before packaging or deployment.".to_string(),
        ],
    }
}

pub fn quality_gate_smoke_check() -> bool {
    let selected =
        DevelopmentMenuTarget::parse_csv("rust,poem,postgresql,graphql,cosmo,cargocheck")
            .unwrap_or_default();
    let plan = build_development_menu_plan(&selected);
    plan.menu_title.contains("aruaru-ai")
        && plan.selected_labels.contains(&"Rust".to_string())
        && plan
            .selected_labels
            .contains(&"Poem / Rust web framework".to_string())
        && plan.selected_labels.contains(&"PostgreSQL".to_string())
        && plan
            .fixed_rules
            .iter()
            .any(|rule| rule.contains("README.md"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_selection_is_allowed() {
        let plan = build_development_menu_plan(&[]);
        assert!(plan.selected_labels.is_empty());
        assert!(plan.selection_rule.contains("none, one, or many"));
    }

    #[test]
    fn multiple_categories_are_supported() {
        let selected =
            DevelopmentMenuTarget::parse_csv("rust,poem,postgresql,graphql,cosmo,chatgpt,gemini")
                .unwrap();
        assert!(selected.contains(&DevelopmentMenuTarget::Rust));
        assert!(selected.contains(&DevelopmentMenuTarget::Poem));
        assert!(selected.contains(&DevelopmentMenuTarget::PostgreSql));
        assert!(selected.contains(&DevelopmentMenuTarget::GraphQl));
        assert!(selected.contains(&DevelopmentMenuTarget::WunderGraphCosmo));
        assert!(selected.contains(&DevelopmentMenuTarget::OpenAiChatGpt));
        assert!(selected.contains(&DevelopmentMenuTarget::GoogleGemini));
    }

    #[test]
    fn menu_covers_required_categories() {
        let plan = build_development_menu_plan(&[]);
        let categories = plan
            .menu_items
            .iter()
            .map(|item| item.category)
            .collect::<Vec<_>>();
        assert!(categories.contains(&DevelopmentCategory::Frontend));
        assert!(categories.contains(&DevelopmentCategory::ProgrammingLanguage));
        assert!(categories.contains(&DevelopmentCategory::BackendFramework));
        assert!(categories.contains(&DevelopmentCategory::FrontendFramework));
        assert!(categories.contains(&DevelopmentCategory::Database));
        assert!(categories.contains(&DevelopmentCategory::ApiStyle));
        assert!(categories.contains(&DevelopmentCategory::AiProvider));
        assert!(categories.contains(&DevelopmentCategory::QualityGate));
    }

    #[test]
    fn fixed_aruaru_rules_are_present() {
        let plan = build_development_menu_plan(&[DevelopmentMenuTarget::Rust]);
        assert!(plan
            .fixed_rules
            .iter()
            .any(|rule| rule.contains("README.md")));
        assert!(plan.fixed_rules.iter().any(|rule| rule.contains("GraphQL")));
        assert!(plan
            .fixed_rules
            .iter()
            .any(|rule| rule.contains("TypeScript")));
    }
}
