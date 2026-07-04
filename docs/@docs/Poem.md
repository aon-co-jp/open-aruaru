# Poem（aruaru HTTP バックエンド）

> 公式: https://docs.rs/poem/latest/poem/  
> GitHub: https://github.com/poem-web/poem

## aruaru での使い方

- エントリ: `src/main.rs`
- ルート: `src/routes/`
- API プレフィックス: `/api/*`
- VersionLess 方針: フィールド追加型で後方互換（`docs/versionless-api.md`）

## よく使うパターン

```rust
use poem::{Route, Server, listener::TcpListener};

let app = Route::new()
    .at("/api/health", poem::get(health))
    .nest("/api", api_routes());
```

## ミドルウェア・OpenAPI

- `poem::middleware::Tracing` — リクエストログ
- `poem-openapi` — OpenAPI スキーマ生成（必要時）

## 公式リンク

- Route: https://docs.rs/poem/latest/poem/struct.Route.html
- Handler: https://docs.rs/poem/latest/poem/trait.Handler.html
- Middleware: https://docs.rs/poem/latest/poem/middleware/index.html
- OpenAPI: https://docs.rs/poem-openapi/latest/poem_openapi/
- Server / TcpListener: https://docs.rs/poem/latest/poem/struct.Server.html

## Cursor での参照

- ローカル: `@docs/@docs/Poem.md`
- @Docs 手動登録名: **Poem** → https://docs.rs/poem/latest/poem/
