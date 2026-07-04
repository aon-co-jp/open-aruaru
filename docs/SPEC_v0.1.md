# aruaru Ver 0.1 — 正式仕様

> **更新**: 2026-06-15  
> **ステータス**: 仕様確定 / 段階的実装中

---

## 1. 概要

**aruaru** は第二の KUSANAGI を目指す、**自己ホスト型・オープンソース** Web 運用プラットフォーム。

| 項目 | 内容 |
|------|------|
| バージョン | **0.1.0** |
| 配布 | GitHub / ZIP → 各自 VPS に FTP + `install aruaru-easyweb` |
| プライバシー | 作者は e-mail を預からない（データは利用者 VPS 内のみ） |

---

## 2. 技術スタック（Ver 0.1 確定）

### フロントエンド

| 技術 | 用途 |
|------|------|
| **HTML5** | マークアップ |
| **CSS3** | スタイル |
| **TypeScript** | クライアントロジック（段階導入） |
| **Bootstrap（2026 モダン UI）** | レスポンシブ・統一デザイン |

### バックエンド

| 技術 | 用途 |
|------|------|
| **Rust** | コアランタイム |
| **Poem** | HTTP サーバー・ルーティング |
| **Tokio** | 非同期ランタイム |
| **SQLx** | DB アクセス（VersionLess Storage 経由） |

### API 統合

| 技術 | 用途 |
|------|------|
| **WunderGraph** | Virtual Graph / TypeScript Operations / クライアント生成 |
| **VersionLess API** | 後方互換を前提とした API 進化（WunderGraph 思想と整合） |

参考（ネット上の公開概念）:

- [Versionless APIs — WunderGraph](https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate)
- [WunderGraph Introduction](https://beta-docs.wundergraph.com/docs/getting-started/introduction)
- [Prisma Data Source（DB → API）](https://docs.wundergraph.com/docs/databases/prisma)

---

## 3. データベース構成

### 3.1 役割分担

```
┌─────────────────────────────────────────────────────────┐
│  VersionLess Storage（Rust trait / SQL 共通方言）          │
├──────────────────────────┬──────────────────────────────┤
│  PostgreSQL（主 DB）      │  CockroachDB Serverless（無料）│
│  読み書きの正             │  自動同期バックアップ           │
│  本番・開発の基準         │  移行先候補・DR               │
└──────────────────────────┴──────────────────────────────┘
```

| DB | 役割 | 備考 |
|----|------|------|
| **PostgreSQL** | **主 DB** | すべての書き込みの正（Source of Truth） |
| **CockroachDB（無料枠）** | **自動同期バックアップ + 移行先候補** | PG 互換。将来マルチリージョン時の切替先 |
| **SQLite** | **開発ブートストラップのみ**（移行期） | v0.1 既存コード互換。本番目標は PostgreSQL |

### 3.2 VersionLess Storage（DB 抽象）

- アプリ・プラグインは **DB エンジン名を意識しない**
- 操作は **ドメイン単位の Storage trait** 経由（`DomainStore`, `AuthStore`, `PluginStore` 等）
- SQL は **PostgreSQL 方言を正** とし、CockroachDB 互換に限定
- **スキーマバージョン** は DB 内 `schema_migrations` で管理（HTTP `/v1` とは独立）

```
migrations/
  common/          # 論理スキーマ（共通）
  postgres/        # PG 固有パッチ
  cockroach/       # CRDB 固有パッチ
  sqlite/          # 開発用ブートストラップ
```

### 3.3 CockroachDB 自動同期（設計）

| 項目 | 方針 |
|------|------|
| 方向 | PostgreSQL → CockroachDB（非同期レプリケーション） |
| 方式 | 定期 CDC / 論理レプリケーション / バッチ sync ジョブ（v0.2 実装） |
| 用途 | バックアップ、フェイルオーバー候補、マルチリージョン移行の準備 |
| 書き込み | **常に PostgreSQL のみ**（split-brain 防止） |

---

## 4. API 二段構え（VersionLess + HTTP 進化）

### 4.1 レイヤー

```
┌──────────────────────────────────────────────────────────┐
│  Browser（HTML5 / CSS3 / TypeScript / Bootstrap 2026）    │
├──────────────────────────────────────────────────────────┤
│  WunderGraph Engine                                       │
│  · Virtual Graph（DB introspection + REST 統合）           │
│  · TypeScript Operations（公開面を明示的に制御）            │
│  · VersionLess: スナップショット + クライアント自動移行     │
├──────────────────────────────────────────────────────────┤
│  Poem HTTP API（/api/*）— 後方互換を意識した進化           │
│  · フィールド追加 OK / 削除は deprecation 期間             │
│  · `/v1` プレフィックスは必須としない                      │
├──────────────────────────────────────────────────────────┤
│  VersionLess Storage（Rust）                              │
├──────────────────────────┬───────────────────────────────┤
│  PostgreSQL              │  CockroachDB（sync replica）   │
└──────────────────────────┴───────────────────────────────┘
```

### 4.2 VersionLess API（WunderGraph 整合）

WunderGraph の **Versionless API** 概念を aruaru に取り込む:

| 概念 | aruaru での適用 |
|------|----------------|
| **Virtual Graph** | PostgreSQL を introspect し、公開 Operations のみ露出 |
| **API スナップショット** | クライアント生成時点のスキーマを記録 |
| **破壊的変更の防止** | デプロイ時に古いクライアント互換を検証 |
| **自動クライアント移行** | フィールド rename 等は migration script で吸収 |
| **JSON-RPC 公開** | GraphQL を直接公開せず、型安全な RPC 端点 |

### 4.3 HTTP API（Poem）ルール

| ルール | 内容 |
|--------|------|
| 追加 | 新フィールド・新エンドポイントは自由 |
| 変更 | 既存フィールドの意味変更は禁止（新フィールドで代替） |
| 削除 | `deprecated: true` を返してから次メジャーで削除 |
| レスポンス | `{ "ok": true, ... }` 形式を維持 |

---

## 5. プラグイン（aruaru-easyweb v0.1）

| プラグイン ID | 機能 |
|---------------|------|
| **aruaru-easyweb** | メタパッケージ（一括 install） |
| **aruaru-domain** | ドメイン追加 |
| **aruaru-vhost** | VirtualHost |
| **aruaru-https** | HTTPS / Let's Encrypt |
| **aruaru-redirect** | リダイレクト |
| **aruaru-proxy-reload** | Apache / Nginx reload |

---

## 6. 認証・プライバシー

- e-mail は **利用者 VPS 内のみ** 保存（作者は預からない）
- マジックリンク: SMTP 未設定時は **画面表示**
- 将来: IP 限定ローカル管理モード（v0.2）

---

## 7. 実装フェーズ

| フェーズ | 内容 | 状態 |
|----------|------|------|
| **0.1a** | Poem + SQLite + easyweb プラグイン + GUI | ✅ 実装済 |
| **0.1b** | VersionLess Storage trait + config 定義 | 🔄 着手 |
| **0.1c** | PostgreSQL 主 DB 切替 + マイグレーション | ✅ 実装済 |
| **0.1d** | WunderGraph 設定 + TS フロント + Bootstrap 2026 | 未着手 |
| **0.2** | CockroachDB 自動同期 + VersionLess クライアント移行 | 未着手 |

---

## 8. 設定（`config/default.toml`）

```toml
[database]
primary = "postgres"           # postgres | sqlite（開発）
postgres_url = "postgres://aruaru:aruaru@127.0.0.1:5432/aruaru"
cockroach_url = ""               # 空 = 同期無効
cockroach_sync_enabled = false
cockroach_sync_interval_secs = 300

[versionless_api]
enabled = true
wundergraph_dir = "wundergraph"
snapshot_on_client_gen = true
```

---

## 9. ディレクトリ（目標構成）

```
aruaru/
├── src/
│   ├── storage/           # VersionLess Storage trait
│   ├── routes/            # Poem HTTP（後方互換進化）
│   └── ...
├── wundergraph/           # WunderGraph SDK 設定
│   ├── wundergraph.config.ts
│   └── operations/
├── frontend/              # TypeScript + Bootstrap 2026（将来）
├── migrations/
│   ├── common/
│   ├── postgres/
│   └── cockroach/
├── plugins/
└── docs/
    └── SPEC_v0.1.md       # ← このファイル
```

---

*aruaru Ver 0.1 正式仕様 — 2026-06-15*
