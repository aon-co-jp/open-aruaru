# Tauri（aruaru デスクトッププラグイン）

> 公式: https://tauri.app/develop/  
> 設定スキーマ: https://v2.tauri.app/reference/config/

## aruaru での使い方

- プラグイン配置: `apps/<plugin>/src-tauri/`
- フロント: TypeScript + Bootstrap（`src/`）
- 命名: `aruaru-<機能>`（例: `aruaru-folder-github`）

## よく使う項目

| 項目 | パス / コマンド |
|------|----------------|
| 設定 | `src-tauri/tauri.conf.json` |
| Rust コマンド | `src-tauri/src/lib.rs` |
| 権限 | `src-tauri/capabilities/default.json` |
| 開発起動 | `npm run tauri dev` |
| ビルド | `npm run tauri build` |

## 公式リンク

- Commands（Rust ↔ JS）: https://tauri.app/develop/calling-rust/
- プラグイン: https://v2.tauri.app/plugin/
- 配布・インストーラー: https://v2.tauri.app/distribute/
- Windows コード署名: https://v2.tauri.app/distribute/sign/windows/

## Cursor での参照

- ローカル: `@docs/@docs/Tauri.md`
- ライブ取得: MCP `fetch` で上記 URL を取得
- @Docs 手動登録名: **Tauri** → https://tauri.app/develop/
