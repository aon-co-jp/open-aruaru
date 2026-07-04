# open-aruaru — データベース設計（DUAL DATABASE）

open-aruaru は **PostgreSQL + aruaru-db** を標準とする **DUAL DATABASE** 戦略を採用する。  
さらに MySQL / SQLite / Redis / MongoDB / ClickHouse など著名な DB を  
オプション機能としてサポートし、プロジェクト用途に応じて組み合わせる。

---

## 1. DUAL DATABASE とは

```
        ┌─────────────────────────────────────────────┐
        │          open-aruaru 各プロジェクト           │
        │  (open-runo / open-e-gov / OpenRedmine …)   │
        └──────────────────┬──────────────────────────┘
                           │ DbBackend trait
              ┌────────────┴──────────────┐
              │       DualBackend         │
              │  (RoutingTable で振り分け)  │
              └──────┬───────────┬────────┘
                     │           │
          ┌──────────▼──┐   ┌───▼──────────────┐
          │ PostgreSQL  │   │    aruaru-db      │
          │  port 5432  │   │    port 5433      │
          │  OLTP/ACID  │   │  Git-on-SQL/Raft  │
          └─────────────┘   └──────────────────┘
```

| 役割 | データベース | ポート | 主な用途 |
|------|-------------|--------|---------|
| **Primary** | PostgreSQL | 5432 | セッション・APIキー・OLTP |
| **Secondary** | aruaru-db | 5433 | スキーマ履歴・監査・Git管理 |

両 DB は **PostgreSQL wire protocol（pgwire）** で接続できるため、  
`sqlx::PgPool` 1 種類のドライバでそのまま扱える。

---

## 2. サポートデータベース全覧

### 2-A. リレーショナル DB（SQL）

| DB | 接続方法 | 用途 | open-aruaru での位置づけ |
|----|---------|------|------------------------|
| **PostgreSQL** 16+ | `sqlx::PgPool` `:5432` | OLTP・汎用本番 | **デフォルト Primary** |
| **MySQL 8** / **MariaDB 11** | `sqlx::MySqlPool` `:3306` | WordPress・OpenRedmine 互換 | OpenWordPress / OpenRedmine の Primary |
| **SQLite 3** | `sqlx::SqlitePool` (file) | ローカル開発・CI・エッジ | dev フォールバック |

### 2-B. 分散 SQL（PostgreSQL / MySQL 互換）

| DB | プロトコル | 用途 | open-aruaru での位置づけ |
|----|-----------|------|------------------------|
| **CockroachDB** | PostgreSQL pgwire | グローバル分散・金融 | aruaru-web クラスタ Secondary |
| **TiDB** | MySQL protocol | 大規模 HTAP | 大規模デプロイ |
| **YugabyteDB** | PostgreSQL pgwire | Geo 分散 | オプション |

CockroachDB・YugabyteDB は pgwire 互換なので `sqlx::PgPool` で接続可能。

### 2-C. NoSQL / ドキュメント型

| DB | クレート | ポート | 用途 | open-aruaru での位置づけ |
|----|---------|--------|------|------------------------|
| **MongoDB** | `mongodb` | 27017 | JSON ドキュメント・検索 | aruaru-crawler / コンテンツ管理 |
| **SurrealDB** | `surrealdb` | 8000 | マルチモデル・Rust ネイティブ | AI データストア候補 |

### 2-D. キー・バリュー / キャッシュ

| DB | クレート | ポート | 用途 | open-aruaru での位置づけ |
|----|---------|--------|------|------------------------|
| **Redis** 7 | `redis` / `deadpool-redis` | 6379 | セッション・レートリミット・PubSub | open-runo レートリミットキャッシュ |
| **KeyDB** | Redis プロトコル互換 | 6379 | Redis 高速互換 | Redis の drop-in 代替 |
| **DragonflyDB** | Redis プロトコル互換 | 6379 | Redis × 25 倍スループット | 大規模キャッシュ |

### 2-E. 列指向 / 分析（OLAP）

| DB | クレート | ポート | 用途 | open-aruaru での位置づけ |
|----|---------|--------|------|------------------------|
| **ClickHouse** | `clickhouse-rs` | 8123/9000 | イベントログ・時系列 | 監視ダッシュボード・AI ログ |
| **DuckDB** | `duckdb` | (embedded) | 高速 OLAP・Parquet | aruaru-db DataFusion と連携 |
| **Apache Parquet** | `parquet2` | — | 列圧縮ファイル形式 | DuckDB / ClickHouse の入力 |

### 2-F. open-aruaru ネイティブ（自社開発）

| DB | 言語 | 接続 | 特徴 |
|----|------|------|------|
| **aruaru-db** | Pure Rust | pgwire `:5433` | Git-on-SQL / Prolly Tree / openraft / DataFusion |

---

## 3. DUAL DATABASE ルーティング戦略

`DualBackend` が論理テーブル名に基づいてバックエンドを選択する。

```
write(table, key, value)
    │
    ▼
RoutingTable::resolve(table)
    │
    ├── "sessions" / "api_keys" / "rate_limits"
    │       → Primary（PostgreSQL）のみ — OLTP 最優先
    │
    ├── "schemas" / "backup_jobs"
    │       → Primary ＋ Secondary 両方（耐久性・冗長性）
    │
    └── "schema_history" / "change_records" / "audit_log"
            → Secondary（aruaru-db）のみ — Git-on-SQL 自動バージョン管理
```

### テーブル→ターゲット対応表

| 論理テーブル | ターゲット | 理由 |
|-------------|-----------|------|
| `sessions` | PostgreSQL | 高頻度 OLTP |
| `api_keys` | PostgreSQL | セキュリティ重視・参照が多い |
| `rate_limits` | PostgreSQL | 超高頻度・低レイテンシ |
| `schemas` | **両方** | 現行スキーマ＋バージョン履歴 |
| `backup_jobs` | **両方** | 耐久性最大化 |
| `schema_history` | aruaru-db | 全変更を Git コミット形式で保存 |
| `change_records` | aruaru-db | 承認フロー・ロールバック対象 |
| `audit_log` | aruaru-db | 不変監査証跡・時系列クエリ |

---

## 4. プロジェクト別 DB 構成推奨

| プロジェクト | Primary DB | Secondary（Git/履歴） | キャッシュ | 分析 |
|------------|-----------|----------------------|-----------|------|
| **open-runo** | PostgreSQL | aruaru-db | Redis | DuckDB |
| **open-e-gov** | PostgreSQL | aruaru-db | Redis | ClickHouse |
| **OpenRedmine** | MySQL / PostgreSQL | aruaru-db | Redis | — |
| **OpenWordPress** | MySQL | aruaru-db | Redis | — |
| **aruaru-llm** | PostgreSQL | aruaru-db | Redis | ClickHouse |
| **aruaru-web** | PostgreSQL | CockroachDB / aruaru-db | Redis | DuckDB |
| **aruaru-crawler** | PostgreSQL | aruaru-db | Redis | MongoDB |
| **local dev** | SQLite | SQLite | — | DuckDB |
| **large cluster** | CockroachDB | aruaru-db | DragonflyDB | ClickHouse |

---

## 5. Rust クレート一覧

### openruno-db feature flags

```toml
[features]
default  = []
# ── リレーショナル ──────────────────────────────
postgres   = ["dep:sqlx", "sqlx/postgres"]   # PostgreSQL 5432
mysql      = ["dep:sqlx", "sqlx/mysql"]      # MySQL/MariaDB 3306
sqlite     = ["dep:sqlx", "sqlx/sqlite"]     # SQLite (file)
aruaru     = ["dep:sqlx", "sqlx/postgres"]   # aruaru-db 5433 (pgwire)
cockroach  = ["dep:sqlx", "sqlx/postgres"]   # CockroachDB (pgwire)
# ── NoSQL ──────────────────────────────────────
mongodb    = ["dep:mongodb"]
surrealdb  = ["dep:surrealdb"]
# ── キャッシュ / KV ─────────────────────────────
redis      = ["dep:redis", "dep:deadpool-redis"]
# ── 分析 ────────────────────────────────────────
clickhouse = ["dep:clickhouse"]
duckdb     = ["dep:duckdb"]
# ── 複合プリセット ───────────────────────────────
dual       = ["postgres", "aruaru"]          # 標準 DUAL DATABASE
full       = ["dual", "redis", "clickhouse"] # 本番フルスタック
dev        = ["sqlite"]                      # ローカル開発・CI
```

### 依存クレート

```toml
[dependencies]
# SQL（feature 制御）
sqlx            = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "chrono", "uuid"], optional = true }
# NoSQL
mongodb         = { version = "3", optional = true }
surrealdb       = { version = "2", optional = true }
# キャッシュ
redis           = { version = "0.27", features = ["tokio-comp", "connection-manager"], optional = true }
deadpool-redis  = { version = "0.18", optional = true }
# 分析
clickhouse      = { version = "0.13", optional = true }
duckdb          = { version = "1", features = ["bundled"], optional = true }
```

---

## 6. DbBackend トレイト

`openruno-db` の `DbBackend` トレイトがすべてのバックエンドを統一する。

```rust
#[async_trait]
pub trait DbBackend: Send + Sync + Debug {
    async fn put(&self, table: &str, key: &str, value: &str) -> Result<()>;
    async fn get(&self, table: &str, key: &str) -> Result<Option<String>>;
    async fn delete(&self, table: &str, key: &str) -> Result<()>;
    async fn list(&self, table: &str) -> Result<Vec<Record>>;
    fn backend_name(&self) -> &'static str;
}
```

各バックエンドはこのトレイトを実装する:

| 実装 | feature | 接続先 |
|------|---------|--------|
| `InMemoryBackend` | (常時) | メモリ（テスト用） |
| `PostgresBackend` | `postgres` | PostgreSQL :5432 |
| `MySqlBackend` | `mysql` | MySQL/MariaDB :3306 |
| `SqliteBackend` | `sqlite` | ファイル or `:memory:` |
| `AruaruDbBackend` | `aruaru` | aruaru-db :5433 (pgwire) |
| `CockroachBackend` | `cockroach` | CockroachDB :26257 (pgwire) |
| `MongoBackend` | `mongodb` | MongoDB :27017 |
| `RedisBackend` | `redis` | Redis :6379 |
| `ClickHouseBackend` | `clickhouse` | ClickHouse :8123 |
| `DualBackend` | `dual` | 上記 2 つを自動振り分け |

---

## 7. キャッシュ戦略（Redis）

Redis は `DbBackend` とは別に **CacheBackend** として利用する。

```
クライアント
    │
    ▼
RedisBackend::get(key)  ←─── hit → 即返却
    │ miss
    ▼
PostgresBackend::get(table, key)
    │
    ▼
RedisBackend::set(key, value, ttl)  ← write-through キャッシュ
    │
    ▼
クライアントへ返却
```

主な用途:
- セッショントークン（TTL 15 分）
- レートリミットカウンター
- GraphQL クエリ結果キャッシュ（open-runo）
- AI プロンプトキャッシュ（aruaru-llm）

---

## 8. 分析戦略（ClickHouse / DuckDB）

```
PostgreSQL / aruaru-db（OLTP）
    │ CDC / バッチ ETL
    ▼
ClickHouse（大規模 OLAP）  ← open-e-gov 監査ログ・イベントログ
    │
    ▼
Grafana / aruaru-web ダッシュボード

aruaru-db（DataFusion 内蔵）
    │ Arrow / Parquet エクスポート
    ▼
DuckDB（軽量 OLAP）  ← ローカル分析・CI レポート
```

---

## 9. aruaru-db について

`aruaru-db` は open-aruaru プロジェクト内で開発中の **Pure Rust 製 Git-on-SQL 分散 DB**。

| 機能 | 詳細 |
|------|------|
| **Git-on-SQL** | Prolly Tree で全データをバージョン管理。`git commit/branch/merge` 相当の SQL 操作が可能 |
| **pgwire 互換** | psql / DBeaver / sqlx がポート 5433 にそのまま接続できる |
| **列指向分析** | Apache Arrow + DataFusion による高速 OLAP クエリ |
| **Raft 分散合意** | `openraft` による強整合クラスタ |
| **Versionless GraphQL** | `async-graphql` によるスキーマ進化対応 |

詳細: [`../../aruaru-db/ARCHITECTURE.md`](../../aruaru-db/ARCHITECTURE.md)

---

## 10. 起動フロー（本番: dual + redis）

```rust
// 1. 接続
let pg     = PostgresBackend::connect(&config.pg_url).await?;
let aruaru = AruaruDbBackend::connect(&config.aruaru_url).await?;
let redis  = RedisBackend::connect(&config.redis_url).await?;

// 2. マイグレーション
migration::postgres::run(pg.pool()).await?;
migration::aruaru::run(aruaru.pool()).await?;

// 3. DualBackend 組み立て
let db = DualBackend::with_default_routing(Arc::new(pg), Arc::new(aruaru));

// 4. AppState に渡す
let state = AppState::with_db(Arc::new(db));
// キャッシュは別途 Arc<RedisBackend> として保持

poem::Server::new(listener).run(build_app(Arc::new(state), rate_limit)).await?;
```

---

## 参照

- [`open-runo/docs/database.md`](../open-runo/docs/database.md) — open-runo 内の詳細設計
- [`open-runo/crates/openruno-db/src/lib.rs`](../open-runo/crates/openruno-db/src/lib.rs) — DbBackend 実装
- [`open-runo/crates/openruno-db/src/dual.rs`](../open-runo/crates/openruno-db/src/dual.rs) — DualBackend 実装
- [`aruaru-db/`](../aruaru-db/) — aruaru-db ソース
