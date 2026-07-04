# aruaru — 将来開発メモ

> **2026-06-15 更新: 既存 aruaru 0.2 は白紙化。コードベースは全削除し、プラグイン構造から作り直す。**

---

## プロダクト方針 — 第二の KUSANAGI（2026-06-15）

### ビジョン

- **オープンソース**の aruaru をダウンロードし、**世界中の VPS / レンタルサーバー**に各自インストールして使う
- 中央 SaaS ではなく **自己ホスト型** — 各利用者のサーバー上で完結
- **開発者（aruaru プロジェクト）は e-mail を預からない** — 収集・保管・送信先管理を一切しない

### 利用者のアクセス方法（利用者が選ぶ）

| 方法 | 例 | 備考 |
|------|-----|------|
| **IP 直アクセス** | `http://160.251.xxx.xxx:3000/` | ドメイン不要、最初の設定に最適 |
| **自分のドメイン** | `https://aruaru.example.com/` | DNS を自分で VPS に向ける |
| **自分のサブドメイン** | `https://panel.example.com/` | 同上 |

### e-mail について（重要）

- メールを **ログイン ID に使う場合** も、データは **その人の VPS 内のみ**（PostgreSQL / 将来 CockroachDB 同期先）
- **aruaru 作者のサーバーには送信されない** / **作者は e-mail を預からない**
- SMTP 設定する場合も **利用者自身のメールサーバー**（任意）
- SMTP 未設定時は **マジックリンクを画面表示**（外部送信なし）
- **将来**: メール登録なしの **IP 限定ローカル管理モード** を追加（Ver 0.2 候補）

### ドメイン運用

- aruaru 本体の **運用ドメイン** は利用者が登録・削除・入れ替え（`platform_domains`）
- **aruaru-https** が HTTP を監視し HTTPS を自動化 — 利用者は「s」を意識しない

### 配布モデル

```
GitHub / 配布 ZIP（オープンソース）
    ↓ ダウンロード
利用者の VPS（ConoHa / ロリポップ / 世界のレンタルサーバー）
    ↓ FTP + install aruaru-easyweb
各自の aruaru が独立稼働（データは各自のサーバーのみ）
```

---

## Ver 0.1 正式仕様（2026-06-15 確定）

> 詳細: **`docs/SPEC_v0.1.md`** / **`docs/versionless-api.md`**

### 技術スタック

| 層 | 技術 |
|----|------|
| フロント | HTML5, CSS3, TypeScript, Bootstrap（2026 モダン UI） |
| API | **WunderGraph** + **VersionLess API** |
| バックエンド | Rust + Poem + Tokio + SQLx |
| DB | **PostgreSQL（主）** + **CockroachDB 無料枠（自動同期バックアップ・移行先候補）** |

### API 二段構え

1. **DB 抽象 — VersionLess Storage** — PG / CRDB / SQLite を trait で統一、スキーマは `schema_migrations` で管理
2. **HTTP — 後方互換を意識した進化** — フィールド追加 OK、削除は deprecation

### WunderGraph VersionLess API

ネット上の公開概念を採用: [Versionless APIs](https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate)

- Virtual Graph + TypeScript Operations
- API スナップショット + クライアント自動移行
- DATABASE も VersionLess Storage で論理バージョン管理

### 実装フェーズ

| フェーズ | 内容 |
|----------|------|
| 0.1a | Poem + SQLite + easyweb（**済**） |
| 0.1b | Storage trait + config（**着手**） |
| 0.1c | PostgreSQL 主 DB |
| 0.1d | WunderGraph + TS フロント |
| 0.2 | CockroachDB 自動同期 |

---

## 作り直し方針（白紙リセット）

| 項目 | 方針 |
|------|------|
| aruaru 0.2 | **破棄** — プロビジョニング等の既存実装は参照しない |
| 新しい起点 | **aruaru-easyweb v0.1** を FTP アップロード → `install aruaru-easyweb` |
| 全体名 | **aruaru** — 全機能のプラットフォーム名 |
| 拡張方式 | 機能ごとに **プラグイン**（install / uninstall / バージョン管理） |

---

## プラグイン構造（新設計）

### プラットフォーム

| 名称 | 役割 |
|------|------|
| **aruaru** | コア — プラグイン管理、重複チェック、AI Gateway（将来） |

### メタパッケージ（最初に配布）

| 名称 | 内容 |
|------|------|
| **aruaru-easyweb** | Web 系プラグイン一式のバンドル（**v0.1 からここを先にインストール**） |

### 個別プラグイン（aruaru-easyweb 配下）

| プラグイン ID | 機能 |
|---------------|------|
| **aruaru-domain** | ドメイン追加 |
| **aruaru-vhost** | VirtualHost 設定 |
| **aruaru-https** | HTTPS 設定、Let's Encrypt 証明書取得・自動更新 |
| **aruaru-redirect** | リダイレクト設定（※ ridirect 表記は使用しない） |
| **aruaru-proxy-reload** | Apache / Nginx 再読み込み |

命名ルール: `aruaru-<機能名>`、小文字・ハイフン統一。

### コア機能（全プラグイン共通）

- **重複チェック** — KUSANAGI、他パネル SSL、certbot 等と機能が被っていないか自動スキャン
- **お知らせ** — 二重設定の恐れを GUI / CLI で通知
- **install / uninstall** — プラグイン単位
- **バージョン管理** — プラグインごとの semver、依存関係

### 初回デプロイ手順（案）

```bash
# 1. レンタルサーバーに aruaru/ フォルダ作成
# 2. aruaru-easyweb v0.1 を FTP アップロード
# 3. インストール
install aruaru-easyweb
# または
aruaru plugin install aruaru-easyweb
```

### プラグインマニフェスト（案）

```toml
# plugins/aruaru-https/manifest.toml
id = "aruaru-https"
version = "0.1.0"
requires = ["aruaru-domain", "aruaru-vhost"]
conflicts = ["kusanagi-ssl", "certbot-standalone"]
provides = ["https", "letsencrypt", "auto-renew"]
```

---

## 保留プロジェクト一覧

| ID | 名称 | 説明 | 優先度 |
|----|------|------|--------|
| wp-rust | WordPress Rust+Poem版 | WPテンプレート・プラグイン選択・契約管理 | 中 |
| redmine-rust | Redmine Rust+Poem版 | プロジェクト管理・チケット | 中 |
| egov-global | e-ガバメント世界政府共通版 | デジタルガバメント基盤 | 低 |
| trade-global | オンライン国際貿易世界共通版 | 貿易・通関連携 | 低 |
| realestate | 不動産売買・賃貸電子契約 | スマホ完結対応 | 低 |
| ai-construction | AI建設会社DX | 建設業向けDX | 低 |
| ai-builder | AI工務店DX | 工務店向けDX | 低 |

---

## 産業別 SAP・マーケットプレイス構想（ユーザーアイデア / 2026-06-15）

> **ステータス**: 構想メモのみ。未実装。Ver 0.3 以降の検討候補。

### 概要

aruaru の **オプション機能**として、産業別の **SAP サンプル**を選択・インストール可能にする。  
AI によるカスタマイズに加え、世界中の業者との **見積・発注・契約** を aruaru 上で完結させるエコシステムを目指す。

### 機能案

#### 1. 産業別 SAP サンプル（オプション）

- 業種・産業ごとに SAP 連携を想定した **サンプルテンプレート**をカタログ化
- GUI で「産業別 SAP サンプル」を選んで **ワンクリックインストール**
- **AI カスタマイズ**: 選択したサンプルを要件に合わせて AI が調整・拡張

#### 2. グローバル業者マーケットプレイス

世界中から以下の業者と **相談・見積依頼** を簡単に行える仕組み:

| 業者タイプ | 例 |
|------------|-----|
| OS メーカー | 各種 OS ベンダー |
| パソコンメーカー | ハードウェア調達 |
| ソフトハウス | 開発・アプリ構築 |
| SIer | システム統合・大規模導入 |

- 見積依頼フォームの標準化（要件・予算・納期・言語）
- 業者プロフィール・実績・対応言語の表示

#### 3. LINE 連携

- 利用者と業者を **LINE アプリ**でつなぐ
- 相談・見積のやり取り、進捗通知、契約前の確認を LINE 経由でも可能に
- （要検討: LINE Messaging API / LINE Official Account / 各国メッセージング代替）

#### 4. 瑕疵保険付き電子契約・注文

- **AFLAC**、**ニッセイ（日本生命）** など保険会社の **瑕疵保険** をセットにした注文・契約フロー
- 電子契約と保険の紐付け（契約内容・保証範囲・保険証券の発行）
- aruaru 上での **注文 → 見積 → 契約 → 保険適用** の一連フロー

### 技術・ビジネス上の検討事項（未着手）

- SAP 連携方式（S/4HANA API、BTP、コネクタ認証）
- 各国の電子契約法・保険商品の法規制対応
- 業者審査・KYC、エスクロー、決済
- LINE 以外の地域別メッセージング（WeChat、WhatsApp 等）
- 瑕疵保険各社との API / 代理店契約
- 多言語・多通貨（有料版 60+ 言語との連携）

### 想定バージョン

| フェーズ | 内容 |
|----------|------|
| Ver 0.3+ | 産業別テンプレートカタログ（SAP サンプル含む）の設計 |
| Ver 0.4+ | 業者マーケットプレイス・見積依頼 MVP |
| Ver 0.5+ | LINE 連携、電子契約、瑕疵保険パートナー連携 |

---

## ~~Ver 0.2~~（破棄）

> aruaru 0.2（FW/DB プロビジョニング等）は白紙化により **参照しない**。以下は履歴のみ。

- ~~FW/DB プロビジョニング~~ → 作り直し後に再検討
- WunderGraph / GraphQL — 将来
- WordPress 連携 — 将来
- SSL — **aruaru-https** プラグインとして再実装
- Docker / Kubernetes — 将来

## 有料版・多言語対応（設計メモ）

有料版では以下を実装予定：

- **60+ 言語対応**: 日本語、英語、台湾語（繁体字中国語）、ペルシャ語、アラビア語、ロシア語、ウクライナ語、ドイツ語、イタリア語、フランス語 など
- **複数言語同時選択**可能
- **支払い方法**: クレジットカード、PayPal、銀行振込、暗号通貨（検討）
- **契約方法**: 月額、年額、従量課金（AIトークン）

## ビジネスモデル

1. **無料版**: 基本機能、PostgreSQL 主 DB、CockroachDB 同期（任意）、VersionLess API
2. **有料版**: 全機能、60+言語、高度AI、商用DBコネクタ
3. **ホストスポンサー**: レンタルサーバー業者が月額支払い → 利用者は全機能無料

## 再開確認

ユーザーが上記プロジェクトの開発を再開したい場合、エージェントは以下を質問してください：

> 「**aruaru-easyweb**（domain / vhost / https / redirect）の開発を再開しますか？」  
> または: WordPress版 / Redmine版 / e-ガバメント / 国際貿易 / 不動産電子契約 / AI建設DX / AI工務店DX / 産業別SAP・業者マーケットプレイス …

---

*更新: 2026-06-15 — 白紙化・プラグイン構造（aruaru-easyweb）*
