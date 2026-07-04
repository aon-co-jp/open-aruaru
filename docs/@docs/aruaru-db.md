# aruaru-db（補助 DB / ダブル DB）

> ソース: `F:\aruaru\aruaru-db`

## aruaru での使い方

- **正（ソース・オブ・トゥルース）**: PostgreSQL
- **補助**: aruaru-db — オフライン同期・ローカルキャッシュ
- Tauri 設定例: `apps/aruaru-dns/src-tauri/config/default.toml` の `source_path`

## 同期方針

1. PostgreSQL へ書き込み
2. 成功後 aruaru-db へミラー
3. 起動時に差分を検出して同期（プラグイン実装による）

## Cursor での参照

- ローカル: `@docs/@docs/aruaru-db.md`
- 関連: `@docs/@docs/PostgreSQL.md`, `@docs/@docs/SQLx.md`
