# aruaru ローカル Docs（@Docs 代替）

Cursor チャットで `@docs/@docs/<名前>.md` と参照する。

| ファイル | 技術 | 公式 URL |
|----------|------|----------|
| [Poem.md](./Poem.md) | HTTP バックエンド | https://docs.rs/poem/latest/poem/ |
| [Rust.md](./Rust.md) | コア言語 | https://doc.rust-lang.org/book/ |
| [Tauri.md](./Tauri.md) | デスクトッププラグイン | https://tauri.app/develop/ |
| [PostgreSQL.md](./PostgreSQL.md) | 本番 DB | https://www.postgresql.org/docs/ |
| [SQLx.md](./SQLx.md) | DB アクセス | https://docs.rs/sqlx/latest/sqlx/ |
| [aruaru-db.md](./aruaru-db.md) | 補助 DB（ダブル DB） | `F:\aruaru\aruaru-db` |
| [WunderGraph.md](./WunderGraph.md) | WunderGraph SDK / Operations | https://docs.wundergraph.com/ |
| [WunderGraph-Cosmo.md](./WunderGraph-Cosmo.md) | GraphQL フェデレーション OSS | https://cosmo-docs.wundergraph.com/ |
| [GraphQL.md](./GraphQL.md) | GraphQL 仕様（参照用） | https://graphql.org/learn/ |
| [VersionLess-API.md](./VersionLess-API.md) | API 進化方針 | WunderGraph ブログ |
| [TypeScript.md](./TypeScript.md) | フロント | https://www.typescriptlang.org/docs/ |
| [Bootstrap.md](./Bootstrap.md) | UI | https://getbootstrap.com/docs/ |

ルール: `.cursor/rules/aruaru-wundergraph-cosmo.mdc`

MCP `fetch` で公式 URL をライブ取得する場合は `.cursor/mcp.json` を有効化。
