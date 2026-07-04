# aruaru-ai Daily Programming Language Update Job

## 1. Problem

The aruaru-ai development menu must not rely on stale fixed text when it explains programming languages. Language ecosystems change: official releases, security warnings, developer surveys, repository trends, and framework popularity all move over time.

The menu must show:

- features
- merits
- demerits
- handoff difficulty
- aruaru-ai recommendation
- source evidence
- confidence
- last updated date
- stale warning
- daily diff history

## 2. Cause

The v18 language information feature defined the language comparison data, but it did not yet define a robust update job with trust ranking, rollback, history, confidence, and UI-safe daily publication.

## 3. Decision

aruaru-ai should run a daily programming language information update job at 03:30 Asia/Tokyo.

The job must crawl only allowlisted public sources, summarize with the selected AI provider, validate output, and publish atomically. Internet evidence and aruaru owner policy must stay separated.

## 4. Trusted source priority

1. Official language documentation and release notes
2. Security and maintenance advisories
3. Major developer surveys such as Stack Overflow Developer Survey
4. Repository ecosystem reports such as GitHub Octoverse
5. Popularity indices such as TIOBE, clearly labeled as popularity only
6. aruaru-ai owner policy, clearly labeled as policy rather than web fact

## 5. Output files

```text
README.md fixed source document
data/language-info/latest.json
data/language-info/history/YYYY-MM-DD.json
data/language-info/diff/YYYY-MM-DD.md
ARUARU_AI_PROGRAMMING_LANGUAGE_INFO.md
```

## 6. Ruby policy

Ruby remains visible in the menu because existing Ruby and Ruby on Rails systems still exist and may require maintenance.

However, aruaru-ai marks Ruby as **NotRecommendedByDefault** for new standard development.

Reason:

- Ruby is expressive and dynamic, so code style can depend strongly on the first developer.
- Japanese documentation is a merit, but documentation volume alone does not guarantee safe handoff.
- Existing Ruby/Rails maintenance is valid, but new aruaru standard development should prefer Rust, TypeScript, Python, or Go unless the user explicitly chooses Ruby.
- If Ruby is selected, aruaru-ai must require tests, conventions, documentation, and handoff notes.

## 7. UI display policy

The aruaru-ai menu should show a compact card for each language:

```text
Language: Rust
Recommendation: StrongDefault
Confidence: 85%
Updated: 2026-06-28
Stale after: 14 days
Main use: core backend / Poem / infrastructure
Merits: safety, performance, reliability
Demerits: learning curve, compile error difficulty
Sources: official docs, survey, ecosystem report
```

For Ruby:

```text
Language: Ruby
Recommendation: NotRecommendedByDefault
Reason: handoff risk for new standard projects
Allowed use: existing Ruby/Rails maintenance only, or explicit user choice
Required safeguards: tests, docs, coding rules, handoff notes
```

## 8. Safety rules

- Do not copy long source text; summarize.
- Do not crawl private repositories, paid docs, or secrets.
- Do not treat popularity as technical superiority.
- Store source metadata and AI summary separately.
- Keep daily history and rollback.
- Publish only after schema validation and secret scan.

## 9. aruaru-ai implementation notes

The production implementation should plug in:

- fetcher
- source allowlist
- AI summarizer
- schema validator
- secret scanner
- history store
- UI badge renderer

The Rust module `language_update_job.rs` defines the testable policy core without requiring network access during `cargo test`.
