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
