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
