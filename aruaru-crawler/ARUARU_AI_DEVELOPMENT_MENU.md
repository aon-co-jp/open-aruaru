# aruaru-ai development menu

## Problem

aruaru-ai needs a standard development menu for app and website creation. The menu must cover frontend, programming languages, frameworks, databases, API/schema choices, AI providers, local LLM runtimes, deployment, package targets, and quality gates.

## Cause

Until v16, the selectable menu was focused on README output targets. That was not enough for real project generation because a project may need several languages, several databases, several AI providers, and several package/deployment targets.

## Decision

README.md is fixed and always generated as the canonical project document. All other development choices are optional checkbox groups. Each group supports:

- no selection
- one selection
- multiple selections

## Checkbox categories

### Frontend / UI

- HTML5 + CSS3 + TypeScript
- Responsive Web / smartphone-tablet-PC-4K
- PWA
- Desktop WebView shell
- Android web package
- iPhone / iPad web package

### Programming language

- Rust
- TypeScript
- Python
- PHP
- Go
- Java
- C# / .NET
- Kotlin
- Swift
- Ruby
- C++
- C
- Zig

### Backend framework

- Poem / Rust web framework
- Axum / Rust web framework
- Actix Web / Rust web framework
- FastAPI / Python
- Django / Python
- Laravel / PHP
- Symfony / PHP
- Gin / Go
- Spring Boot / Java
- ASP.NET Core / C#
- Ktor / Kotlin
- Ruby on Rails

### Frontend framework

- Vanilla TypeScript
- React
- Vue
- Svelte
- Solid
- Angular
- Astro
- Next.js
- Nuxt

### Database / storage

- PostgreSQL
- CockroachDB
- SQLite
- MySQL
- MariaDB
- MongoDB
- Redis
- ClickHouse
- DuckDB
- Elasticsearch
- OpenSearch
- S3-compatible object storage

### API / schema

- GraphQL
- WunderGraph Cosmo
- gRPC
- WebSocket
- SSE / Server-Sent Events

### AI provider

- ChatGPT / OpenAI API
- Claude / Opus / Anthropic API
- Gemini / Google API
- DeepSeek API

### Local LLM runtime

- aruaru-llm local runtime
- OpenCUDA project runtime
- Ollama
- LM Studio
- GGUF local model

### DevOps / deployment

- Docker
- Podman
- Kubernetes
- systemd
- Nginx
- Apache httpd
- Caddy
- Let's Encrypt / Certbot
- ConoHa VPS
- GitHub Actions
- Redmine report export

### Package target

- Windows ZIP package
- Linux tar.gz package
- macOS app bundle
- Android APK/AAB package
- iOS package
- Web deploy folder

### Quality gate

- cargo fmt
- cargo check
- cargo test
- cargo clippy -- -D warnings
- PowerShell bug check script
- Secret / API key scan
- README generation check

## Fixed aruaru rules

- README.md is always generated.
- Rust + Poem remains the default aruaru core stack.
- TypeScript is preferred over plain JavaScript.
- Internal aruaru APIs should prefer GraphQL and WunderGraph Cosmo, not internal REST.
- Tauri is not selected by default and is not part of the aruaru standard stack.
- API keys, `.env`, SSH keys, tokens, and certificates must never be written to generated menus, README files, logs, or frontend output.

## Implementation

The Rust module is:

```text
src/development_menu.rs
```

Important functions:

```text
DevelopmentMenuTarget::parse_csv(...)
build_development_menu_plan(...)
quality_gate_smoke_check()
```

This keeps the menu connected to the binary so `cargo clippy --all-targets -- -D warnings` does not treat it as dead code.


## v19 Daily Programming Language Update Job

v19 adds `src/language_update_job.rs` and `ARUARU_AI_DAILY_LANGUAGE_UPDATE_JOB.md`.

The aruaru-ai programming language menu now has a daily update-job design:

- crawl allowlisted public language sources once per day
- separate web evidence from aruaru owner policy
- summarize features, merits, demerits, and handoff risks
- show confidence and stale warnings
- keep `latest.json`, daily history, and diff markdown
- keep Ruby visible for existing Ruby/Rails maintenance, but mark it `NotRecommendedByDefault` for new standard development
