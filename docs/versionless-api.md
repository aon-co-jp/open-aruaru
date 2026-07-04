# VersionLess API — aruaru 採用方針

aruaru Ver 0.1 では、**WunderGraph** が提唱する **Versionless API** の考え方を仕様に含める。

## 参照（ネット上の公開資料）

| 資料 | URL |
|------|-----|
| Versionless APIs（本体） | https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate |
| WunderGraph 概要 | https://beta-docs.wundergraph.com/docs/getting-started/introduction |
| DB → API（Prisma introspection） | https://docs.wundergraph.com/docs/databases/prisma |
| Backend agnostic / Realtime | https://wundergraph.com/blog/instant_realtime_apis_using_postgresql_mysql_sqlite_sqlserver_planetscale |

## aruaru での二段構え

### 1. DB 層 — VersionLess Storage

- PostgreSQL / CockroachDB /（開発用 SQLite）を **同じ trait** で操作
- スキーマは `schema_migrations` テーブルで **論理バージョン管理**
- HTTP の `/v1` とは **独立**

### 2. HTTP 層 — 後方互換を意識した進化

- Poem `/api/*` は **フィールド追加型** で進化
- WunderGraph 側は **スナップショット + クライアント自動移行** で VersionLess を実現

## WunderGraph との相性

| WunderGraph の強み | aruaru への効果 |
|-------------------|----------------|
| Virtual Graph | PostgreSQL を introspect し Operations のみ公開 |
| TypeScript Operations | DB スキーマを直接クライアントに晒さない |
| クライアント生成 | TypeScript フロントに型安全 API |
| Versionless デプロイ | プラグイン・フロント更新時の破壊的変更を抑止 |
| Backend agnostic | dev=SQLite / prod=PostgreSQL も同一 Operations で可能 |

## DATABASE の VersionLess バージョン管理

```
論理スキーマ v3  ──→  PostgreSQL（正）
                 ──→  CockroachDB（sync replica, 同一論理 v3）
                 ──→  WunderGraph Virtual Graph（introspect v3）
```

- **論理バージョン**: `migrations/common/` の連番
- **物理差分**: `postgres/` / `cockroach/` / `sqlite/` で吸収
- **API バージョン**: URL に `/v1` を付けず、スナップショットで管理

詳細は [SPEC_v0.1.md](./SPEC_v0.1.md) を参照。
