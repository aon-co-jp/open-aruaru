# PostgreSQL（aruaru 本番 DB）

> 公式: https://www.postgresql.org/docs/

## aruaru での使い方

- マイグレーション: `migrations/postgres/`
- 接続: `config/default.toml` `[database]`
- SQLx 経由: `src/storage/pool.rs`
- ダブル DB: PostgreSQL（正）+ aruaru-db（`F:\aruaru\aruaru-db`）

## 公式リンク

- SQL 構文: https://www.postgresql.org/docs/current/sql-commands.html
- データ型: https://www.postgresql.org/docs/current/datatype.html
- ON CONFLICT: https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT

## Cursor での参照

- ローカル: `@docs/@docs/PostgreSQL.md`
- @Docs 手動登録名: **PostgreSQL** → https://www.postgresql.org/docs/
