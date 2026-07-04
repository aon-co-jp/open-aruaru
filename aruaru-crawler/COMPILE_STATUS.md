# Compile status v10

このChatGPT実行環境では cargo / rustc を実行できないため、実コンパイル確認は未実施です。

前回ユーザー環境での結果:

```text
OK: full bug check passed
```

今回のv10で追加:

- src/hybrid_theory.rs
- ARUARU_LLM_HYBRID_FOLDING_SBM_THEORY.md
- ARUARU_VOLUNTEER_GT730_CONTEXT.md
- README.md 追記

zip作成前に実施した確認:

- required files check
- Rust source brace balance rough check
- banned Tauri marker check
- REST / REST API implementation marker check
- README update check
- zip integrity check

ローカル確認コマンド:

```powershell
cd F:\aruaru\aruaru-rs4

powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
cargo test hybrid_theory
cargo test llm_folding
cargo test quasi_quantum
cargo test api_key_handoff
cargo check
cargo clippy --all-targets -- -D warnings
```


## v11 manual COPY script generator

この環境では cargo / rustc を実行できないため、実コンパイル確認は未実施です。

追加確認:

- `src/copy_script_generator.rs` を追加
- `main.rs` に `copy_script_generator` module を追加
- PowerShell COPYテンプレートを追加
- Bash COPYテンプレートを追加
- COPY利用説明書を追加
- Rustソース括弧バランス粗チェック
- src/Cargo.toml の禁止仕様混入チェック

推奨確認コマンド:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
cargo test copy_script_generator
cargo test manual_script_generator
cargo test hybrid_theory
cargo test llm_folding
cargo test quasi_quantum
cargo test api_key_handoff
cargo check
cargo clippy --all-targets -- -D warnings
```


# v12 check notes

修正内容:

- `scripts/manual/aruaru-manual-copy-template.ps1` の `param(...)` を最初の実行文に移動
- `src/copy_script_generator.rs` のPowerShell生成テンプレートも同じ修正を適用
- 生成テストに `param(...)` が `$ErrorActionPreference` より前にあることを確認するassertを追加

この環境では `cargo / rustc / powershell` を実行できないため、実コンパイルとPowerShell実行確認は未実施です。

## v13 manual copy boolean parser fix

Fixed PowerShell manual copy script parameters so `powershell -File ... -DryRun $true` and string-style values such as `true`, `false`, `1`, and `0` are normalized safely.

Reason: Windows PowerShell can pass arguments after `-File` as strings. A strict `[bool]$DryRun` parameter may fail before the script body runs.

Recommended command:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun true
```


## v14 manual COPY script variable-colon fix

Windows PowerShellでは、二重引用符内の `$Name:` がドライブ名付き変数のように解釈されるため、`${Name}:` に修正しました。

前回ユーザー環境での基準結果:

```text
OK: full bug check passed
```

この環境では cargo / rustc / PowerShell 実行は未実施です。
確認済み:

- manual COPY PowerShell template parse-risk fix
- Rust generator template fix
- zip integrity check
- source brace balance rough check
- banned Tauri / REST API implementation marker rough check


## v15 fix

- Connected `api_key_handoff` through a runtime quality-gate smoke check so `cargo clippy --all-targets -- -D warnings` does not fail on dead code.
- Added startup README scan so an existing `README.md` is converted immediately to `README.rs` / `README.html`.
- Strengthened `scripts/check-full.ps1` generation test by rewriting `README.md` after server startup and capturing cargo stdout/stderr.


## v16 status

This environment still cannot run `cargo` / `rustc`, so real compilation is not verified here.

Added quality targets:

```powershell
cargo test readme_output_menu
cargo test generator
cargo clippy --all-targets -- -D warnings
```

Expected full check:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
```

The full check now verifies README.rs, README.html, README.php, README.py, README.ts, and README.json generation.


## v17 status

Added `src/development_menu.rs` and connected it from `main.rs` through `development_menu::quality_gate_smoke_check()`. The goal is to prevent dead-code warnings under `cargo clippy --all-targets -- -D warnings` while making the aruaru-ai development menu a standard feature.

Expected checks on Windows:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
cargo test development_menu
cargo check
cargo clippy --all-targets -- -D warnings
```


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
