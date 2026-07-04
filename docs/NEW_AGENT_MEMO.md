# aruaru — New Agent 引き継ぎ MEMO

> **ワークスペース**: `e:\aruaru`  
> **更新**: 2026-06-15  
> **正式仕様**: `docs/SPEC_v0.1.md`（必読）

---

## 1. プロジェクト概要

**aruaru Ver 0.1** — 第二の KUSANAGI を目指す **自己ホスト型 Web 運用プラットフォーム**（オープンソース）。

| 項目 | 内容 |
|------|------|
| バージョン | 0.1.0 |
| フロント | HTML5, CSS3, TypeScript, Bootstrap 2026 |
| バックエンド | **Rust + Poem + Tokio + SQLx** |
| API | **WunderGraph + VersionLess API** |
| DB | **PostgreSQL（主）** + **CockroachDB 無料枠（同期バックアップ・移行先）** |
| ターゲット | ConoHa VPS / ロリポップ / 世界中のレンタルサーバー |
| インストール | FTP → `deploy/install.sh` / `aruaru install aruaru-easyweb` |

### プライバシー

- 作者は **e-mail を預からない** — データは利用者 VPS 内のみ
- IP 直アクセス or 自分のドメイン / サブドメイン

---

## 2. アーキテクチャ（Ver 0.1 確定）

### API 二段構え

| 層 | 方針 |
|----|------|
| **VersionLess Storage** | DB 抽象 — PostgreSQL / CockroachDB / SQLite（dev） |
| **HTTP `/api/*`** | 後方互換を意識した進化（`/v1` 必須ではない） |
| **WunderGraph** | Virtual Graph, TypeScript Operations, Versionless クライアント |

参照: `docs/versionless-api.md`  
[WunderGraph Versionless APIs](https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate)

### DB 構成

```
PostgreSQL（主 DB・書き込みの正）
    ↓ 自動同期（v0.2）
CockroachDB Serverless 無料枠（バックアップ + 移行先候補）
```

---

## 3. 実装済み（0.1a）

- [x] Rust + Poem Web サーバー（port 3000）
- [x] SQLite / PostgreSQL 切替（`[database] primary`）
- [x] マイグレーション（`migrations/postgres/` / `migrations/sqlite/`）
- [x] メール ID + マジックリンク認証
- [x] aruaru-easyweb プラグイン（domain / vhost / https / redirect / proxy-reload）
- [x] platform_domains（本体ドメイン登録・入れ替え）
- [x] HTTPS 監視スタブ
- [x] 定期リマインダー（FUTURE_MEMO）
- [x] `deploy/install.sh` + systemd

---

## 4. 着手・未実装

| フェーズ | 内容 | 状態 |
|----------|------|------|
| **0.1b** | VersionLess Storage trait + config | ✅ 済 |
| **0.1c** | PostgreSQL 主 DB + マイグレーション | ✅ 済 |
| **0.1d** | WunderGraph + TypeScript + Bootstrap 2026 | 未 |
| **0.2** | CockroachDB 自動同期 | 未 |
| **0.2** | certbot / Let's Encrypt 本実装 | 未 |

---

## 5. ディレクトリ構成

```
e:\aruaru/
├── src/
│   ├── storage/mod.rs       # VersionLess Storage trait
│   ├── state.rs             # DB・設定
│   ├── platform_domain.rs
│   ├── routes/api.rs        # Poem HTTP API
│   └── plugins/
├── wundergraph/             # WunderGraph 設定（準備中）
├── migrations/              # common / postgres / cockroach（予定）
├── plugins/                 # プラグインマニフェスト
├── config/default.toml
├── docs/
│   ├── SPEC_v0.1.md         # 正式仕様
│   ├── versionless-api.md
│   └── FUTURE_MEMO.md
└── deploy/
```

---

## 6. API 一覧（Poem）

| Method | Path | 説明 |
|--------|------|------|
| GET | `/api/health` | ヘルスチェック |
| POST | `/api/register` | 初回登録 |
| POST | `/api/login` | ログインリンク |
| GET/POST | `/api/domains` | サイトドメイン |
| GET/POST | `/api/platform/domains` | 本体運用ドメイン |
| GET | `/api/plugins` | プラグイン一覧 |
| POST | `/api/install` | プラグイン install |

GUI: `/register`, `/login`, `/dashboard`, `/settings/easyweb`, `/settings/domain`, `/settings/https`

---

## 7. Cursor / ConoHa VPS

```powershell
.\deploy\cursor-vps.ps1 status   # VPS + aruaru 状態
.\deploy\cursor-vps.ps1 check    # JSON（エージェント用）
.\deploy\cursor-vps.ps1 connect  # SSH
```

設定: `config/vps.toml` / タスク: **VPS: 状態確認**

```bash
cd e:\aruaru
cargo build
cargo run serve
# http://127.0.0.1:3000/register
```

環境変数: `ARUARU_DATA_DIR`（デフォルト `./data`）

---

## 8. ユーザー方針

1. **Rust + Poem** がメイン、KUSANAGI は使わない
2. **VersionLess API**（WunderGraph 思想）を仕様に含める
3. **PostgreSQL 主 DB** + **CockroachDB 同期バックアップ**
4. オープンソース配布、各自 VPS に設置
5. **git commit は明示依頼時のみ**

---

## 9. New Agent プロンプト例

```
e:\aruaru の aruaru Ver 0.1 を継続開発。
docs/SPEC_v0.1.md と docs/NEW_AGENT_MEMO.md を読んでから [タスク] を実行。
```

---

*更新: 2026-06-15 — Ver 0.1 仕様確定（VersionLess API / PostgreSQL / CockroachDB）*
