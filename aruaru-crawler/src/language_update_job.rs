//! Daily programming-language information update job for aruaru-ai.
//!
//! This module is intentionally network-client agnostic. The production aruaru-ai
//! desktop/web app can plug in a fetcher, an AI summarizer, and a persistence
//! adapter. The core rules stay testable without internet access: source
//! priority, freshness windows, confidence scoring, Ruby policy handling, and
//! rollback-safe publication.

use serde::{Deserialize, Serialize};

use crate::programming_language_info::{
    build_default_language_crawler_plan, LanguageRecommendation, LanguageSourceKind,
    ProgrammingLanguageInfo,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrawlFrequency {
    Daily,
    Weekly,
    ManualOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTrustLevel {
    Highest,
    High,
    Medium,
    Policy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedCrawlSource {
    pub name: String,
    pub url: String,
    pub kind: LanguageSourceKind,
    pub trust_level: SourceTrustLevel,
    pub frequency: CrawlFrequency,
    pub evidence_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageSummarySnapshot {
    pub language: String,
    pub recommendation: LanguageRecommendation,
    pub confidence_percent: u8,
    pub features: Vec<String>,
    pub merits: Vec<String>,
    pub demerits: Vec<String>,
    pub handoff_risks: Vec<String>,
    pub source_count: usize,
    pub stale_after_days: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyLanguageUpdatePlan {
    pub job_name: String,
    pub schedule_local_time: String,
    pub output_files: Vec<String>,
    pub trusted_sources: Vec<TrustedCrawlSource>,
    pub update_pipeline: Vec<String>,
    pub rollback_rules: Vec<String>,
    pub snapshots: Vec<LanguageSummarySnapshot>,
}

fn source_trust(kind: LanguageSourceKind) -> SourceTrustLevel {
    match kind {
        LanguageSourceKind::OfficialDocumentation => SourceTrustLevel::Highest,
        LanguageSourceKind::SecurityOrMaintenanceReport => SourceTrustLevel::High,
        LanguageSourceKind::EcosystemSurvey => SourceTrustLevel::High,
        LanguageSourceKind::RepositoryTrend => SourceTrustLevel::Medium,
        LanguageSourceKind::PopularityIndex => SourceTrustLevel::Medium,
        LanguageSourceKind::InternalAruaruPolicy => SourceTrustLevel::Policy,
    }
}

fn source_frequency(kind: LanguageSourceKind) -> CrawlFrequency {
    match kind {
        LanguageSourceKind::InternalAruaruPolicy => CrawlFrequency::ManualOnly,
        _ => CrawlFrequency::Daily,
    }
}

fn confidence_for(info: &ProgrammingLanguageInfo) -> u8 {
    let official = info
        .sources
        .iter()
        .filter(|source| source.kind == LanguageSourceKind::OfficialDocumentation)
        .count();
    let external = info
        .sources
        .iter()
        .filter(|source| source.kind != LanguageSourceKind::InternalAruaruPolicy)
        .count();
    let policy = info
        .sources
        .iter()
        .any(|source| source.kind == LanguageSourceKind::InternalAruaruPolicy);

    let mut score = 45_u8;
    score = score.saturating_add((official.min(2) as u8) * 20);
    score = score.saturating_add((external.min(3) as u8) * 8);
    if policy {
        score = score.saturating_add(6);
    }
    score.min(95)
}

fn stale_after_days(info: &ProgrammingLanguageInfo) -> u16 {
    match info.aruaru_recommendation {
        LanguageRecommendation::StrongDefault => 14,
        LanguageRecommendation::Recommended => 14,
        LanguageRecommendation::Situational => 30,
        LanguageRecommendation::LegacyOrSpecialist => 60,
        LanguageRecommendation::NotRecommendedByDefault => 30,
    }
}

pub fn build_daily_language_update_plan() -> DailyLanguageUpdatePlan {
    let crawler = build_default_language_crawler_plan();
    let mut trusted_sources = Vec::new();

    for info in &crawler.languages {
        for source in &info.sources {
            if trusted_sources
                .iter()
                .any(|item: &TrustedCrawlSource| item.url == source.url)
            {
                continue;
            }
            trusted_sources.push(TrustedCrawlSource {
                name: source.title.clone(),
                url: source.url.clone(),
                kind: source.kind,
                trust_level: source_trust(source.kind),
                frequency: source_frequency(source.kind),
                evidence_note: match source.kind {
                    LanguageSourceKind::OfficialDocumentation => {
                        "Use for language facts, supported versions, install guidance, and official positioning.".to_string()
                    }
                    LanguageSourceKind::EcosystemSurvey => {
                        "Use for developer usage, admiration, and pain-point trends; never treat as absolute truth.".to_string()
                    }
                    LanguageSourceKind::PopularityIndex => {
                        "Use only as popularity signal; never use as technical quality ranking.".to_string()
                    }
                    LanguageSourceKind::RepositoryTrend => {
                        "Use as repository activity signal and compare with survey / official data.".to_string()
                    }
                    LanguageSourceKind::SecurityOrMaintenanceReport => {
                        "Use as high-priority risk signal when choosing default stacks.".to_string()
                    }
                    LanguageSourceKind::InternalAruaruPolicy => {
                        "Use as project-owner policy; keep separate from internet evidence.".to_string()
                    }
                },
            });
        }
    }

    let snapshots = crawler
        .languages
        .iter()
        .map(|info| LanguageSummarySnapshot {
            language: info.language.clone(),
            recommendation: info.aruaru_recommendation,
            confidence_percent: confidence_for(info),
            features: info.features.clone(),
            merits: info.merits.clone(),
            demerits: info.demerits.clone(),
            handoff_risks: info.handoff_notes.clone(),
            source_count: info.sources.len(),
            stale_after_days: stale_after_days(info),
        })
        .collect();

    DailyLanguageUpdatePlan {
        job_name: "aruaru-ai daily programming language information update".to_string(),
        schedule_local_time: "03:30 Asia/Tokyo daily".to_string(),
        output_files: vec![
            "data/language-info/latest.json".to_string(),
            "data/language-info/history/YYYY-MM-DD.json".to_string(),
            "data/language-info/diff/YYYY-MM-DD.md".to_string(),
            "ARUARU_AI_PROGRAMMING_LANGUAGE_INFO.md".to_string(),
        ],
        trusted_sources,
        update_pipeline: vec![
            "Fetch only allowlisted public sources with ETag and Last-Modified caching.".to_string(),
            "Extract title, URL, retrieval date, publication date, and short factual evidence.".to_string(),
            "Ask the selected AI provider to summarize features, merits, demerits, handoff risks, and recommendation impact.".to_string(),
            "Separate internet evidence from aruaru owner policy; do not pretend policy is a web fact.".to_string(),
            "Apply contradiction rules: official docs beat blogs; security reports beat popularity indices; owner policy may override default recommendation.".to_string(),
            "Validate JSON schema, run secret scan, keep previous version, and publish atomically.".to_string(),
        ],
        rollback_rules: vec![
            "Never overwrite latest.json unless validation succeeds.".to_string(),
            "Keep daily history so a bad AI summary can be rolled back.".to_string(),
            "Show confidence and stale markers in the aruaru-ai menu.".to_string(),
            "When sources disagree, show both signals instead of hiding the conflict.".to_string(),
        ],
        snapshots,
    }
}

pub fn daily_language_update_markdown() -> String {
    let plan = build_daily_language_update_plan();
    let mut out = String::new();
    out.push_str("# aruaru-ai Daily Programming Language Update Job\n\n");
    out.push_str(&format!("Schedule: {}\n\n", plan.schedule_local_time));

    out.push_str("## Update pipeline\n");
    for step in &plan.update_pipeline {
        out.push_str(&format!("- {}\n", step));
    }

    out.push_str("\n## Language snapshots\n");
    for item in &plan.snapshots {
        out.push_str(&format!(
            "- {}: {:?}, confidence {}%, stale after {} days\n",
            item.language, item.recommendation, item.confidence_percent, item.stale_after_days
        ));
    }

    out.push_str("\n## Ruby policy\n");
    out.push_str("Ruby remains visible for existing Ruby/Rails maintenance, but aruaru-ai marks it NotRecommendedByDefault for new standard development unless the user explicitly selects it and accepts handoff discipline.\n");
    out
}

pub fn quality_gate_smoke_check() -> bool {
    let plan = build_daily_language_update_plan();
    let ruby = plan.snapshots.iter().find(|item| item.language == "Ruby");
    let rust = plan.snapshots.iter().find(|item| item.language == "Rust");

    plan.schedule_local_time.contains("Asia/Tokyo")
        && plan
            .output_files
            .iter()
            .any(|item| item.contains("latest.json"))
        && plan
            .rollback_rules
            .iter()
            .any(|rule| rule.contains("rollback"))
        && ruby.is_some_and(|item| {
            item.recommendation == LanguageRecommendation::NotRecommendedByDefault
        })
        && rust.is_some_and(|item| item.confidence_percent >= 80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_update_uses_tokyo_schedule_and_history() {
        let plan = build_daily_language_update_plan();
        assert!(plan.schedule_local_time.contains("Asia/Tokyo"));
        assert!(plan
            .output_files
            .iter()
            .any(|item| item.contains("history")));
        assert!(plan
            .output_files
            .iter()
            .any(|item| item.contains("latest.json")));
    }

    #[test]
    fn ruby_remains_not_recommended_by_default() {
        let plan = build_daily_language_update_plan();
        let ruby = plan
            .snapshots
            .iter()
            .find(|item| item.language == "Ruby")
            .unwrap();
        assert_eq!(
            ruby.recommendation,
            LanguageRecommendation::NotRecommendedByDefault
        );
        assert!(ruby
            .handoff_risks
            .iter()
            .any(|item| item.contains("do not recommend Ruby by default")));
    }

    #[test]
    fn official_sources_are_highest_trust() {
        let plan = build_daily_language_update_plan();
        assert!(plan.trusted_sources.iter().any(|source| {
            source.kind == LanguageSourceKind::OfficialDocumentation
                && source.trust_level == SourceTrustLevel::Highest
        }));
    }

    #[test]
    fn markdown_contains_confidence_and_ruby_policy() {
        let markdown = daily_language_update_markdown();
        assert!(markdown.contains("confidence"));
        assert!(markdown.contains("Ruby policy"));
        assert!(markdown.contains("NotRecommendedByDefault"));
    }
}
