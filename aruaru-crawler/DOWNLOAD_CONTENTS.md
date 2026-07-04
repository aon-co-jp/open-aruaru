# ダウンロード内容

このZIPには、現在の修正済みプログラムと関連システムDATAを含めています。

## プログラム

- Rust + Poem サーバーアプリ
- README.md 監視
- README.rs 自動生成
- README.html 自動生成
- スマホ縦横、タブレット、PC、WQHD、4K、8K、16K想定レスポンシブHTML

## 修正済みBUG

- `dashboard` 二重定義BUG
- Poem `EndpointExt` import不足BUG
- `scripts` フォルダから実行した時の相対パスBUG
- PowerShellスクリプト内日本語文字化けBUG

## システムDATA / 設計資料

- GT730環境BUGチェック手順
- aruaru-desktop PowerShell自動操作・自動BUGチェック仕様
- aruaru-desktop + aruaru-ai + aruaru-llm + iLumi.llm 連動仕様
- ROADMAP
- systemd service例
- Windows PowerShellチェックスクリプト
- Linux/macOS用チェックスクリプト

## 次回実行

```powershell
cd F:\aruaru\aruaru-rs\aruaru-rs3.1
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
```


## v6追加

- `src/llm_folding.rs` — DeepSeek Folding Engine の実験的プランナー
- `ARUARU_LLM_DEEPSEEK_FOLDING.md` — DeepSeek系公開技術の調査分析とaruaru実装方針
- `ARUARU_LLM_TOSHIBA_QUASI_QUANTUM_NEXT.md` — 次回向け疑似量子インスパイア最適化メモ
- `README.md` — aruaru-llm DeepSeek Folding Engine 追記


## v7 Toshiba SBM-Inspired additions

- `src/quasi_quantum.rs`
- `ARUARU_LLM_TOSHIBA_SBM_ENGINE.md`
- `ARUARU_LLM_NEXT_HYBRID_THEORY.md`
- `RUN_CHECK_COMMANDS_WINDOWS.md`

v7 adds a Toshiba SBM-inspired quantum optimization planner for aruaru-llm. It is an experimental QUBO-style optimizer for bug-fix order, test order, model routing, prompt compression, and README generation planning.

## v8追加

```text
src/manual_script_generator.rs
ARUARU_WEB_DESKTOP_SCRIPT_GENERATOR.md
scripts/manual/README_USAGE.md
scripts/manual/aruaru-manual-bugcheck-template.ps1
scripts/manual/aruaru-manual-bugcheck-template.sh
```

追加内容:

- aruaru-desktop / aruaru-web 連携仕様
- ブラウザから利用可能な aruaru-ai 自動BUGチェック仕様
- 手動BUGチェックスクリプト生成機能
- 手動スクリプト利用方法説明書


## v9 API KEY Handoff additions

- `src/api_key_handoff.rs`
- `ARUARU_AI_API_KEY_HANDOFF.md`
- `scripts/manual/API_KEY_HANDOFF_USAGE.md`
- `scripts/manual/aruaru-api-key-import-template.ps1`

Purpose: safely import API keys for API-based LLMs such as Claude Opus, OpenAI / ChatGPT, Gemini, and DeepSeek without exposing keys in logs, README files, generated frontend files, or Git.

## v10 Hybrid Folding + SBM-Inspired update

Added:

```text
src/hybrid_theory.rs
ARUARU_LLM_HYBRID_FOLDING_SBM_THEORY.md
ARUARU_VOLUNTEER_GT730_CONTEXT.md
```

README.md was updated with:

- DeepSeek Folding Engine review
- Toshiba SBM-Inspired Optimizer review
- Hybrid Folding + SBM Fusion
- GT730 / no-salary volunteer / self-funded operating context
- Safety notes that avoid unsupported performance claims

Recommended checks:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
cargo test hybrid_theory
cargo test llm_folding
cargo test quasi_quantum
cargo test api_key_handoff
cargo check
cargo clippy --all-targets -- -D warnings
```


## v11 manual COPY script generator update

Added:

```text
src/copy_script_generator.rs
ARUARU_DESKTOP_COPY_SCRIPT_GENERATOR.md
scripts/manual/COPY_SCRIPT_USAGE.md
scripts/manual/aruaru-manual-copy-template.ps1
scripts/manual/aruaru-manual-copy-template.sh
```

Purpose:

- aruaru-desktop / aruaru-ai standard manual COPY script generation
- dry-run first workflow
- secret-file exclusion
- no-delete safe copy
- Windows PowerShell and Bash templates

Recommended checks:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
cargo test copy_script_generator
cargo test manual_script_generator
cargo test hybrid_theory
cargo check
cargo clippy --all-targets -- -D warnings
```


## v12 added / fixed

- Fixed manual COPY PowerShell parser bug.
- Updated copy script generator so future generated scripts also put `param(...)` first.
- Updated manual COPY usage documentation.

## v13 update

- Fixed `scripts/manual/aruaru-manual-copy-template.ps1` boolean parameter parsing.
- Updated `src/copy_script_generator.rs` so newly generated PowerShell copy scripts use robust boolean normalization.
- Added tests that check the generated script includes `[object]$DryRun` and `Convert-AruaruBool`.


## v14 additions

- Fixed `scripts/manual/aruaru-manual-copy-template.ps1` PowerShell parse error caused by `$Name:` inside a double-quoted string.
- Fixed `src/copy_script_generator.rs` so generated scripts use `${Name}:` safely.
- Added notes that previous stable baseline was `OK: full bug check passed`.


## v15 fix

- Connected `api_key_handoff` through a runtime quality-gate smoke check so `cargo clippy --all-targets -- -D warnings` does not fail on dead code.
- Added startup README scan so an existing `README.md` is converted immediately to `README.rs` / `README.html`.
- Strengthened `scripts/check-full.ps1` generation test by rewriting `README.md` after server startup and capturing cargo stdout/stderr.


## v16 additions

- `src/readme_output_menu.rs`
- aruaru-ai README output checkbox menu
- `--extra-outputs` support
- Optional generation for README.rs, README.html, README.php, README.py, README.ts, README.js, README.go, README.java, README.cs, README.kt, README.swift, README.rb, README.json
- `ARUARU_AI_README_OUTPUT_MENU.md`


## v17 added files

```text
src/development_menu.rs
ARUARU_AI_DEVELOPMENT_MENU.md
```

The new module defines the aruaru-ai development menu with checkbox groups for frontend, programming languages, frameworks, databases, API/schema, AI providers, local LLM runtimes, DevOps, package targets, and quality gates.


## v18 - aruaru-ai Programming Language Information Crawler

Added language-by-language feature / merit / demerit / handoff guidance for the aruaru-ai development menu.
README.md remains fixed, and programming language choices now show daily-refreshable guidance with source metadata.

Ruby policy: Ruby is recognized as a Japanese-created language with many Japanese resources, but aruaru-ai does not recommend Ruby by default for new multi-developer projects because handoff can fail when implicit style and original-developer intent dominate the codebase.

New files:

- src/programming_language_info.rs
- ARUARU_AI_PROGRAMMING_LANGUAGE_INFO.md

Quality gates:

- cargo test programming_language_info
- cargo check
- cargo clippy --all-targets -- -D warnings


## v19 Daily Programming Language Update Job

v19 adds `src/language_update_job.rs` and `ARUARU_AI_DAILY_LANGUAGE_UPDATE_JOB.md`.

The aruaru-ai programming language menu now has a daily update-job design:

- crawl allowlisted public language sources once per day
- separate web evidence from aruaru owner policy
- summarize features, merits, demerits, and handoff risks
- show confidence and stale warnings
- keep `latest.json`, daily history, and diff markdown
- keep Ruby visible for existing Ruby/Rails maintenance, but mark it `NotRecommendedByDefault` for new standard development


## v20 TOP100 Programming Languages

Added `src/language_top100.rs` and `ARUARU_AI_TOP100_PROGRAMMING_LANGUAGES.md`. aruaru-ai now has a daily-updated TOP100 programming language report with features, merits, demerits, author/governance, author policy/claim, and aruaru recommendation. Popularity signals are separated from technical recommendation. Ruby remains visible but NotRecommendedByDefault for new standard development.
