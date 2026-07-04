# 技術スタック選択ガイド（aruaru / Cursor エージェント用）

開発・BUG 修正依頼時、エージェントが Yes/No で確認するための参照資料。  
公式 Docs は `docs/@docs/` または MCP fetch で自動参照する。

---

## aruaru 既定スタック（Ver 0.1）

| 層 | 技術 | ローカル Docs |
|----|------|---------------|
| 言語 | Rust | `docs/@docs/Rust.md` |
| HTTP | Poem + Tokio | `docs/@docs/Poem.md` |
| DB | PostgreSQL + SQLx | `docs/@docs/PostgreSQL.md`, `SQLx.md` |
| 補助 DB | aruaru-db（`F:\aruaru\aruaru-db`） | リポジトリ README |
| API | VersionLess + **WunderGraph Cosmo** + WunderGraph SDK | `WunderGraph-Cosmo.md`, `WunderGraph.md`, `VersionLess-API.md` |
| フロント | TypeScript + Bootstrap | `docs/@docs/TypeScript.md`, `Bootstrap.md` |
| デスクトップ | Tauri 2 | `docs/@docs/Tauri.md` |

---

## API 層（WunderGraph Cosmo + Cursor）

| 項目 | 方針 |
|------|------|
| GraphQL 基盤 | **WunderGraph Cosmo**（OSS・自前ホスト可） |
| Operations / TS | **WunderGraph SDK**（`wundergraph/`） |
| VersionLess 思想 | 維持 — `versionless-api.md` |
| DB ハイブリッド | `schema_migrations` + SQLx + aruaru-db |
| 管理画面 | Cosmo Studio + **不足は Cursor で TS + Bootstrap 開発** |
| Cursor ルール | `.cursor/rules/aruaru-wundergraph-cosmo.mdc` |

---

### Rust

| 項目 | 内容 |
|------|------|
| 人気の層 | インフラ・組み込み・高性能 Web・セキュリティ重視のチーム |
| 産業 | クラウド、ゲーム、ブロックチェーン、組み込み、自社 VPS 運用 |
| 引き継ぎ | **◎** 型 + コンパイラが仕様を強制。コメントが少なくても意図が残りやすい |
| 初期速度 | △ 学習曲線は急。慣れれば大規模でも破綻しにくい |
| 大規模 | ◎ メモリ安全・並行処理向き |
| AI 学習量 | 中〜多（増加中） |
| Docs 登録の価値 | **高** — Poem 等は AI 単体より公式 Docs 参照が効く |
| 公式 | https://doc.rust-lang.org/book/ |

### Python

| 項目 | 内容 |
|------|------|
| 人気の層 | データ分析、ML、スタートアップ、社内ツール |
| 産業 | AI/研究、フィンテック、教育、自動化 |
| 引き継ぎ | **○〜◎** 可読性高い。型ヒント + テストがあるとさらに良い |
| 初期速度 | **◎** プロトタイプ最速クラス |
| 大規模 | ○ ディシプリン次第（型・分割・テスト必須） |
| AI 学習量 | **非常に多** |
| Docs 登録の価値 | 中（AI だけでもだいたい動く） |
| 公式 | https://docs.python.org/3/ |

### Ruby

| 項目 | 内容 |
|------|------|
| 人気の層 | Web スタートアップ、受託の短期納品 |
| 産業 | EC、メディア、国内 Web 受託 |
| 引き継ぎ | **△** Rails の「慣習・魔法」に依存しやすく、**最初は速いが他人引き継ぎが難しい**ことが多い |
| 初期速度 | **◎** CRUD・管理画面は最速クラス |
| 大規模 | ○ 設計・モジュール分割が必須 |
| AI 学習量 | 多 |
| Docs 登録の価値 | 中 |
| 公式 | https://guides.rubyonrails.org/ |

### TypeScript

| 項目 | 内容 |
|------|------|
| 人気の層 | フロントエンド、フルスタック、Node バックエンド |
| 産業 | SaaS、広く全産業 |
| 引き継ぎ | **◎** 型があり、フロント人材プール最大 |
| 初期速度 | ○ |
| 大規模 | **◎** |
| AI 学習量 | **非常に多** |
| Docs 登録の価値 | 中 |
| 公式 | https://www.typescriptlang.org/docs/ |

### Go

| 項目 | 内容 |
|------|------|
| 人気の層 | インフラ、CLI、マイクロサービス |
| 産業 | DevOps、クラウドネイティブ |
| 引き継ぎ | **◎** シンプルな文法、チーム統一しやすい |
| 初期速度 | ○ |
| 大規模 | ○ |
| AI 学習量 | 多 |
| 公式 | https://go.dev/doc/ |

---

## バックエンドフレームワーク

### Poem（Rust）— aruaru 既定

| 項目 | 内容 |
|------|------|
| 特徴 | 軽量 HTTP、async、OpenAPI 対応 |
| 引き継ぎ | **◎** Rust の型とセットでメンテしやすい |
| 大規模 | ○〜◎ |
| AI + Docs | AI 情報は少なめ → **`docs/@docs/Poem.md` + fetch 必須** |
| 公式 | https://docs.rs/poem/latest/poem/ |

### FastAPI（Python）

| 項目 | 内容 |
|------|------|
| 特徴 | 型ヒント、OpenAPI 自動生成、非同期 |
| 引き継ぎ | **◎** コードが短く読みやすい。**チームメンテ向き** |
| 大規模 | ○（レイヤ分割が必要） |
| AI + Docs | **情報豊富** |
| 公式 | https://fastapi.tiangolo.com/ |

### Ruby on Rails

| 項目 | 内容 |
|------|------|
| 特徴 | 規約駆動、CRUD 最速 |
| 引き継ぎ | **△** 暗黙の慣習に依存 |
| 大規模 | ○（モノリス分割が課題） |
| 公式 | https://guides.rubyonrails.org/ |

### Actix-web / Axum（Rust）

| 項目 | 内容 |
|------|------|
| 特徴 | 高性能 Rust HTTP（Poem の代替） |
| 引き継ぎ | ◎（Rust 共通） |
| 公式 | https://docs.rs/axum/latest/axum/ |

---

## フロントエンド

### TypeScript + Bootstrap — aruaru 既定

| 項目 | 内容 |
|------|------|
| 人気 | 管理画面・業務アプリ・自社ホスト UI |
| 引き継ぎ | **◎** |
| 公式 | Bootstrap https://getbootstrap.com/docs/ |

### React / Vue

| 項目 | 内容 |
|------|------|
| 人気 | SPA、大規模 SaaS |
| 引き継ぎ | ◎（エコシステム大） |
| 大規模 | **◎** |
| トレードオフ | ビルド複雑、aruaru 0.1 より重い |

---

## データベース

### PostgreSQL — aruaru 本番既定

| 項目 | 内容 |
|------|------|
| 人気 | 業務システム、自社 VPS、オープンソース基盤 |
| 引き継ぎ | **◎** 標準 SQL、人材多い |
| 大規模 | **◎** |
| 公式 | https://www.postgresql.org/docs/ |

### SQLite

| 項目 | 内容 |
|------|------|
| 用途 | 開発・単体・エッジ |
| 引き継ぎ | ◎ |
| aruaru | dev フォールバック |

### CockroachDB

| 項目 | 内容 |
|------|------|
| 用途 | aruaru 0.2 同期バックアップ候補 |
| 大規模 | ◎ 分散向き |

---

## API / 統合

### VersionLess API + WunderGraph — aruaru 既定

| 項目 | 内容 |
|------|------|
| 特徴 | 後方互換 API、GraphQL、TS クライアント生成 |
| 引き継ぎ | ◎ スキーマ・型で契約が明確 |
| 公式 | https://docs.wundergraph.com/ |
| 思想 | `docs/versionless-api.md` |

### REST のみ（Poem `/api/*`）

| 項目 | 内容 |
|------|------|
| 特徴 | シンプル、既存 aruaru 実装と一致 |
| 引き継ぎ | ◎ |

---

## デスクトップ

### Tauri 2 — aruaru プラグイン既定

| 項目 | 内容 |
|------|------|
| 特徴 | Rust バックエンド + Web UI、軽量、インストーラー |
| 人気 | クロスプラットフォーム、Electron より軽量派 |
| 引き継ぎ | ○（Rust + TS の両方が必要） |
| AI + Docs | **Docs 登録で将来性に賭けやすい** |
| 公式 | https://tauri.app/develop/ |

### Electron

| 項目 | 内容 |
|------|------|
| 特徴 | Node + Chromium、実績最大 |
| 引き継ぎ | ◎（JS 人材多い） |
| デメリット | サイズ・メモリ大 |

---

## 新しい技術 + AI の考え方

| 状況 | 推奨 |
|------|------|
| AI 学習データが少ない（例: Poem, 新しい FW） | `docs/@docs/` + MCP fetch + Cursor @Docs で **公式を毎回参照** |
| 引き継ぎ重視 | Rust+Poem / Python+FastAPI / TypeScript — **型・テスト・Docs** |
| 最速プロトタイプ | Ruby / Python / PHP |
| 大規模・長期 VPS 運用 | **Rust + Poem + PostgreSQL**（aruaru 方針） |

---

## エージェント用 AskQuestion テンプレート

### スタック全体

- **Yes** — aruaru 既定で進める
- **No** — カテゴリ別に選択

### カテゴリ例（No 時）

**言語:** Rust / Python / TypeScript / Ruby / Go / その他  
**バックエンド:** Poem / FastAPI / Rails / Axum / その他  
**DB:** PostgreSQL / SQLite / CockroachDB / その他  
**フロント:** TS+Bootstrap / React / Vue  
**API:** VersionLess+WunderGraph / REST のみ  
**デスクトップ:** Tauri / なし / Electron  

選択後、対応する `docs/@docs/*.md` を開いてから実装する。
