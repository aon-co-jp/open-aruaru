# Rust / Cargo が無い場合の導入メモ

このChatGPT実行コンテナでは `rustc` / `cargo` は見つかりませんでした。ローカルPCで確認する場合は、Rust公式の rustup で導入してください。

Windows PowerShell:

```powershell
winget install Rustlang.Rustup
rustup default stable
rustc --version
cargo --version
```

Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

確認:

```bash
cd aruaru-platform
cargo check --workspace
cargo test --workspace
```
