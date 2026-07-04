#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceClass {
    PublicCompanyFiling,
    OfficialCaseStudy,
    MarketSurvey,
    EngineeringArticle,
    AcademicOrPatent,
    CreatorClaim,
    UnverifiedOrSpeculative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLabel {
    Practical,
    Watch,
    HighRisk,
    Speculative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusinessTop100Item {
    pub rank: usize,
    pub category: &'static str,
    pub name: &'static str,
    pub why_profitable: &'static str,
    pub likely_stack: &'static str,
    pub features: &'static str,
    pub merits: &'static str,
    pub demerits: &'static str,
    pub author_claim_or_policy: &'static str,
    pub evidence: EvidenceClass,
    pub risk: RiskLabel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlPolicy {
    pub schedule_local_time: &'static str,
    pub top100_groups: Vec<&'static str>,
    pub source_priority: Vec<&'static str>,
    pub output_files: Vec<&'static str>,
    pub safety_rules: Vec<&'static str>,
}

pub fn daily_business_top100_crawl_policy() -> CrawlPolicy {
    CrawlPolicy {
        schedule_local_time: "03:45 Asia/Tokyo daily",
        top100_groups: vec![
            "profitable web services and marketplaces",
            "e-commerce, real estate, construction, contractor services",
            "telemedicine, TV chat, remote consultation, online education",
            "enterprise order systems, SAP, intranet, bespoke apps",
            "new power plants plus AI data centers",
            "PC, tablet, smartphone, GPU, AI and LLM products",
            "DDC, USB-DAC, amplifier, speaker and audio technology",
            "databases, frameworks, VersionlessAPI and schema evolution",
            "new theories, quantum-inspired optimization, folding and material science",
            "creator claims and YouTube hypotheses with strict verification labels",
        ],
        source_priority: vec![
            "official company filings and investor relations",
            "official engineering blogs and architecture case studies",
            "official documentation for languages, frameworks, databases and APIs",
            "government, standards, medical and energy regulators",
            "academic papers, patents and reputable technical reports",
            "market surveys and developer surveys",
            "news articles and product reviews",
            "YouTube creator claims, always separated from verified facts",
        ],
        output_files: vec![
            "data/business_top100/latest.json",
            "data/business_top100/history/YYYY-MM-DD.json",
            "data/business_top100/diff/YYYY-MM-DD.md",
            "reports/business_top100.md",
            "reports/business_top100_redmine.md",
        ],
        safety_rules: vec![
            "separate facts, estimates, aruaru-ai interpretation and creator claims",
            "never present revenue estimates as guaranteed profit",
            "mark medical, energy and investment topics as high-stakes verification required",
            "mark air-to-gold and free-oil style claims as unverified unless strong evidence exists",
            "avoid scraping private or paywalled content without permission",
            "store source URL, access date, confidence score and rollback history",
        ],
    }
}

fn make_item(
    rank: usize,
    category: &'static str,
    name: &'static str,
    stack: &'static str,
) -> BusinessTop100Item {
    BusinessTop100Item {
        rank,
        category,
        name,
        why_profitable: "network effects, recurring demand, operational automation, data advantage, payment flow or enterprise switching cost",
        likely_stack: stack,
        features: "tracks business model, traffic, users, monetization, moat, operations and technology choices",
        merits: "helps aruaru-ai propose realistic stacks based on profitable real-world patterns",
        demerits: "public information may be incomplete; revenue, margin and stack details require confidence labels",
        author_claim_or_policy: "aruaru-ai separates verified facts, estimates, user policy and creator claims before recommending a stack",
        evidence: EvidenceClass::MarketSurvey,
        risk: RiskLabel::Watch,
    }
}

pub fn business_top100_seed_catalog() -> Vec<BusinessTop100Item> {
    let seeds: [(&str, &str, &str); 100] = [
        (
            "E-commerce",
            "Amazon-style marketplace",
            "Java, Kotlin, Rust, TypeScript, DynamoDB, Aurora, search, event streaming",
        ),
        (
            "E-commerce",
            "Rakuten-style mall",
            "Java, PHP, TypeScript, PostgreSQL, MySQL, search, payment services",
        ),
        (
            "E-commerce",
            "Shopify-style merchant platform",
            "Ruby legacy, Go, Rust, TypeScript, MySQL, Kafka-like events",
        ),
        (
            "E-commerce",
            "Mercari-style C2C marketplace",
            "Go, TypeScript, Swift, Kotlin, PostgreSQL, ML moderation",
        ),
        (
            "E-commerce",
            "Digital content marketplace",
            "TypeScript, Rust, Go, PostgreSQL, object storage, GraphQL",
        ),
        (
            "Real estate",
            "Real estate listing portal",
            "TypeScript, Java, Python, PostgreSQL, Elasticsearch, map services",
        ),
        (
            "Real estate",
            "Brokerage CRM and lead marketplace",
            "TypeScript, C#, Java, PostgreSQL, BI warehouse",
        ),
        (
            "Real estate",
            "Property management SaaS",
            "TypeScript, Java, PostgreSQL, accounting integration",
        ),
        (
            "Real estate",
            "Construction quote matching",
            "TypeScript, Rust, Poem, PostgreSQL, GraphQL",
        ),
        (
            "Real estate",
            "Rental screening and contract automation",
            "TypeScript, Python, PostgreSQL, document AI",
        ),
        (
            "Construction",
            "Custom home builder portal",
            "Rust, TypeScript, PostgreSQL, file workflow, approval graph",
        ),
        (
            "Construction",
            "Contractor scheduling marketplace",
            "Go, TypeScript, PostgreSQL, mobile apps",
        ),
        (
            "Construction",
            "BIM and estimate integration",
            "C#, TypeScript, PostgreSQL, object storage",
        ),
        (
            "Construction",
            "Materials procurement platform",
            "Java, TypeScript, PostgreSQL, EDI, inventory sync",
        ),
        (
            "Construction",
            "Maintenance subscription service",
            "TypeScript, Python, PostgreSQL, routing optimization",
        ),
        (
            "Healthcare",
            "TV chat telemedicine",
            "TypeScript, WebRTC, Rust/Go backend, PostgreSQL, audit logs",
        ),
        (
            "Healthcare",
            "Remote elderly care consultation",
            "TypeScript, Kotlin, Swift, PostgreSQL, video, alerts",
        ),
        (
            "Healthcare",
            "Hospital appointment SaaS",
            "Java, C#, PostgreSQL, HL7/FHIR integration",
        ),
        (
            "Healthcare",
            "Pharmacy delivery workflow",
            "TypeScript, Go, PostgreSQL, logistics integration",
        ),
        (
            "Healthcare",
            "AI triage assistant",
            "Python, Rust, TypeScript, vector DB, strict medical review",
        ),
        (
            "Education",
            "Online programming school",
            "TypeScript, Rust, PostgreSQL, LLM tutoring, progress graph",
        ),
        (
            "Education",
            "English conversation app",
            "TypeScript, Kotlin, Swift, PostgreSQL, speech API",
        ),
        (
            "Education",
            "Exam simulator with certificate",
            "TypeScript, Rust, PostgreSQL, PDF generation",
        ),
        (
            "Communication",
            "TV chat service",
            "TypeScript, WebRTC, Go/Rust, Redis, PostgreSQL",
        ),
        (
            "Communication",
            "Live commerce",
            "TypeScript, WebRTC, payments, PostgreSQL, CDN",
        ),
        (
            "Enterprise",
            "SAP extension and integration",
            "Java, ABAP integration, TypeScript, PostgreSQL, event bus",
        ),
        (
            "Enterprise",
            "Custom intranet order system",
            "Rust, Poem, TypeScript, PostgreSQL, GraphQL",
        ),
        (
            "Enterprise",
            "Workflow approval system",
            "C#, Java, TypeScript, PostgreSQL, audit trail",
        ),
        (
            "Enterprise",
            "Document AI and search",
            "Python, Rust, TypeScript, OpenSearch, vector DB",
        ),
        (
            "Enterprise",
            "ERP replacement for SMEs",
            "Rust/Go, TypeScript, PostgreSQL, accounting rules",
        ),
        (
            "AI data center",
            "New power plant plus AI data center",
            "Rust, Go, Kubernetes, telemetry DB, energy optimization",
        ),
        (
            "AI data center",
            "GPU cluster rental",
            "Python, Go, Rust, Kubernetes, billing, Prometheus",
        ),
        (
            "AI data center",
            "Sovereign AI cloud",
            "Rust, Go, Kubernetes, PostgreSQL, object storage, model registry",
        ),
        (
            "AI data center",
            "Waste heat reuse service",
            "Go, Rust, telemetry, IoT, time-series DB",
        ),
        (
            "AI data center",
            "Demand response AI",
            "Python, Rust, time-series DB, optimization engine",
        ),
        (
            "AI/LLM",
            "LLM coding assistant",
            "TypeScript, Rust, Python, vector DB, GraphQL",
        ),
        (
            "AI/LLM",
            "Local LLM desktop manager",
            "Rust, TypeScript, GGUF, model registry",
        ),
        (
            "AI/LLM",
            "Agent workflow platform",
            "TypeScript, Python, PostgreSQL, queue, browser automation",
        ),
        (
            "AI/LLM",
            "RAG knowledge base",
            "Python, Rust, PostgreSQL, vector DB, object storage",
        ),
        (
            "AI/LLM",
            "AI call center",
            "TypeScript, Python, speech, PostgreSQL, CRM integration",
        ),
        ("Languages", "Rust", "Rust, Poem/Axum, PostgreSQL, GraphQL"),
        (
            "Languages",
            "TypeScript",
            "TypeScript, HTML5, CSS3, GraphQL, Node-compatible tooling",
        ),
        (
            "Languages",
            "Python",
            "Python, FastAPI, AI libraries, PostgreSQL",
        ),
        (
            "Languages",
            "Go",
            "Go, Gin/Fiber, PostgreSQL, microservices",
        ),
        (
            "Languages",
            "PHP",
            "PHP, Laravel, MySQL/PostgreSQL, CMS integrations",
        ),
        ("Frameworks", "Poem", "Rust, Poem, async, GraphQL adapters"),
        (
            "Frameworks",
            "Axum",
            "Rust, Tower, async services, PostgreSQL",
        ),
        (
            "Frameworks",
            "FastAPI",
            "Python, Pydantic, OpenAPI, AI prototypes",
        ),
        (
            "Frameworks",
            "Laravel",
            "PHP, Eloquent, queues, MySQL/PostgreSQL",
        ),
        (
            "Frameworks",
            "Spring Boot",
            "Java/Kotlin, enterprise integration, PostgreSQL/Oracle",
        ),
        (
            "Database",
            "PostgreSQL",
            "PostgreSQL, JSONB, extensions, GraphQL integration",
        ),
        (
            "Database",
            "CockroachDB",
            "PostgreSQL-compatible distributed SQL",
        ),
        (
            "Database",
            "SQLite",
            "embedded local data, edge apps, tests",
        ),
        ("Database", "MySQL/MariaDB", "web hosting, CMS, e-commerce"),
        (
            "Database",
            "MongoDB",
            "document data, flexible schemas, app prototypes",
        ),
        (
            "Database",
            "Redis/Valkey",
            "cache, queue, sessions, real-time state",
        ),
        (
            "Database",
            "ClickHouse",
            "analytics, logs, BI, high-volume events",
        ),
        (
            "Database",
            "DuckDB",
            "local analytics, CSV/Parquet analysis",
        ),
        (
            "Database",
            "OpenSearch/Elasticsearch",
            "search, log analysis, discovery",
        ),
        (
            "Database",
            "Vector database",
            "RAG, semantic search, embeddings",
        ),
        (
            "API",
            "VersionlessAPI",
            "GraphQL, schema evolution, deprecation policy",
        ),
        (
            "API",
            "WunderGraph Cosmo",
            "GraphQL federation, schema governance",
        ),
        (
            "API",
            "GraphQL",
            "typed schema, client-driven fields, compatibility control",
        ),
        (
            "API",
            "gRPC",
            "strong contracts, high-throughput internal systems",
        ),
        (
            "API",
            "WebSocket/SSE",
            "real-time chat, progress, notifications",
        ),
        (
            "Devices",
            "PC sales and BTO configuration",
            "TypeScript, PostgreSQL, recommendation engine",
        ),
        (
            "Devices",
            "Tablet and smartphone comparison",
            "TypeScript, crawler, review scoring, affiliate workflow",
        ),
        (
            "Devices",
            "GPU benchmark and resale intelligence",
            "Python, Rust, PostgreSQL, price crawler",
        ),
        (
            "Devices",
            "AI PC service",
            "Rust, TypeScript, local LLM installer, hardware detection",
        ),
        (
            "Devices",
            "Repair and upgrade marketplace",
            "TypeScript, PostgreSQL, scheduling and parts inventory",
        ),
        (
            "Audio",
            "DDC comparison site",
            "TypeScript, PostgreSQL, review crawler, measurement DB",
        ),
        (
            "Audio",
            "USB-DAC ranking",
            "TypeScript, Python, audio measurements, PostgreSQL",
        ),
        (
            "Audio",
            "Amplifier review and sales funnel",
            "TypeScript, PostgreSQL, long-form content",
        ),
        (
            "Audio",
            "Speaker comparison service",
            "TypeScript, Python, measurement, subjective review separation",
        ),
        (
            "Audio",
            "Room acoustic measurement service",
            "Rust, Python DSP, PostgreSQL, charts",
        ),
        (
            "Energy",
            "Small modular reactor watch",
            "Regulatory crawler, Python, PostgreSQL, risk labels",
        ),
        (
            "Energy",
            "Geothermal plus data center",
            "IoT telemetry, time-series DB, optimization",
        ),
        (
            "Energy",
            "Solar, battery and AI workload scheduling",
            "Python, Rust, time-series DB",
        ),
        (
            "Energy",
            "Hydrogen and e-fuel watch",
            "Crawler, chemistry sources, confidence labels",
        ),
        (
            "Energy",
            "Fusion power watch",
            "Academic/official crawler, maturity score",
        ),
        (
            "New theory",
            "Toshiba SBM-inspired optimization",
            "Rust optimizer, QUBO-like planner, job routing",
        ),
        (
            "New theory",
            "DeepSeek Folding-inspired compression",
            "prompt compression, routing, distillation, log folding",
        ),
        (
            "New theory",
            "Versionless schema evolution",
            "GraphQL compatibility rules, deprecation automation",
        ),
        (
            "New theory",
            "Molecular control and materials discovery",
            "AI chemistry, simulation, lab verification required",
        ),
        (
            "New theory",
            "Air-to-fuel",
            "CO2 capture, hydrogen, catalysis, energy accounting",
        ),
        (
            "Speculative",
            "Air-to-gold claims",
            "unverified creator claim, nuclear transmutation reality check",
        ),
        (
            "Speculative",
            "Free-energy YouTube claim",
            "unverified creator claim, physics validation required",
        ),
        (
            "Speculative",
            "Miracle battery claim",
            "patent and lab replication required",
        ),
        (
            "Speculative",
            "Room-temperature superconductivity claim",
            "peer review and replication required",
        ),
        (
            "Speculative",
            "AI auto-builds entire company claim",
            "prototype evidence and legal/compliance check required",
        ),
        (
            "Business model",
            "Subscription SaaS",
            "TypeScript, Rust/Go, PostgreSQL, billing",
        ),
        (
            "Business model",
            "Marketplace fee model",
            "TypeScript, PostgreSQL, search, payment escrow",
        ),
        (
            "Business model",
            "Advertising and affiliate model",
            "crawler, SEO, TypeScript, analytics DB",
        ),
        (
            "Business model",
            "Enterprise license model",
            "Java/C#/Rust, PostgreSQL, SSO, audit",
        ),
        (
            "Business model",
            "Usage-based AI API billing",
            "Rust/Go, PostgreSQL, metering, fraud detection",
        ),
        (
            "Operations",
            "Customer success automation",
            "TypeScript, CRM integration, LLM summarization",
        ),
        (
            "Operations",
            "Price intelligence crawler",
            "Python/Rust, PostgreSQL, scheduler",
        ),
        (
            "Operations",
            "Lead scoring engine",
            "Python, TypeScript, PostgreSQL, ML features",
        ),
        (
            "Operations",
            "Compliance and audit trail",
            "Rust/C#, PostgreSQL, immutable logs",
        ),
        (
            "Operations",
            "Redmine and project report automation",
            "Rust, Poem, PostgreSQL, Markdown reports",
        ),
    ];

    seeds
        .iter()
        .enumerate()
        .map(|(index, (category, name, stack))| make_item(index + 1, category, name, stack))
        .collect()
}

pub fn classify_youtube_claim(claim: &str) -> RiskLabel {
    let lower = claim.to_ascii_lowercase();
    if lower.contains("air") && (lower.contains("gold") || lower.contains("oil")) {
        return RiskLabel::Speculative;
    }
    if lower.contains("free energy") || lower.contains("miracle") {
        return RiskLabel::Speculative;
    }
    RiskLabel::Watch
}

pub fn quality_gate_smoke_check() -> usize {
    let policy = daily_business_top100_crawl_policy();
    let catalog = business_top100_seed_catalog();
    assert!(policy.schedule_local_time.contains("Asia/Tokyo"));
    assert_eq!(catalog.len(), 100);
    assert_eq!(
        classify_youtube_claim("air to gold"),
        RiskLabel::Speculative
    );
    catalog.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_exactly_top100_items() {
        let items = business_top100_seed_catalog();
        assert_eq!(items.len(), 100);
        assert_eq!(items.first().unwrap().rank, 1);
        assert_eq!(items.last().unwrap().rank, 100);
    }

    #[test]
    fn policy_keeps_claims_separated_from_facts() {
        let policy = daily_business_top100_crawl_policy();
        assert!(policy
            .safety_rules
            .iter()
            .any(|rule| rule.contains("separate facts")));
        assert!(policy
            .source_priority
            .iter()
            .any(|source| source.contains("official company filings")));
    }

    #[test]
    fn speculative_youtube_claims_are_not_treated_as_facts() {
        assert_eq!(
            classify_youtube_claim("air to gold machine"),
            RiskLabel::Speculative
        );
        assert_eq!(
            classify_youtube_claim("free energy miracle device"),
            RiskLabel::Speculative
        );
    }

    #[test]
    fn includes_required_business_and_technology_topics() {
        let names: Vec<&str> = business_top100_seed_catalog()
            .into_iter()
            .map(|item| item.name)
            .collect();
        assert!(names.contains(&"Amazon-style marketplace"));
        assert!(names.contains(&"TV chat telemedicine"));
        assert!(names.contains(&"New power plant plus AI data center"));
        assert!(names.contains(&"SAP extension and integration"));
        assert!(names.contains(&"USB-DAC ranking"));
        assert!(names.contains(&"VersionlessAPI"));
    }
}
