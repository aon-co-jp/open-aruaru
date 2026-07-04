# VersionLess API（aruaru 仕様）

> 思想（WunderGraph）: https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate

## aruaru 二段構え

1. **DB 層** — VersionLess Storage（`schema_migrations` で論理バージョン）
2. **HTTP 層** — Poem `/api/*` はフィールド追加型で進化

詳細: `docs/versionless-api.md`, `docs/SPEC_v0.1.md`

## Cursor での参照

- ローカル: `@docs/@docs/VersionLess-API.md`
- @Docs 手動登録名: **VersionLess-API** → 上記ブログ URL
