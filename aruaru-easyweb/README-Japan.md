# aruaru-easyweb

**第二の KUSANAGI** — Rust + Poem による VPS Web サーバー管理アプリ

---

## 概要

aruaru-easyweb は、VPS 上の Web サーバー設定をブラウザから完全に管理できるアプリです。  
VI / nano などのエディタ操作は一切不要。すべて GUI で完結します。

---

## 主な機能

### 🔐 Magic Link ログイン（パスワードレス）
- IP アドレスでブラウザにアクセスするだけで起動
- メールアドレスを入力 → ワンタイムログインリンクをメール送信
- リンクは 15 分で自動失効
- 毎回メールアドレスを入力するシンプルセキュリティ

### 🌐 ドメイン・サブドメイン管理
- **bare ドメインと www を自動で両対応**（例: `example.com` と `www.example.com`）
- www → 非 www への自動リダイレクト
- aruaru-easyweb 自身のドメイン登録・変更も可能
- ドメイン追加・削除・変更がすべて GUI で完結

### 🔒 HTTPS 自動管理（Let's Encrypt）
- ドメイン登録時に証明書を **自動取得**（certbot 連携）
- bare ドメイン + www の両方を同時取得
- 残り 30 日以内になると **自動更新**（毎日 03:00 スケジューラ実行）
- ダッシュボードで証明書の残り日数をリアルタイム表示

### ⚙️ Nginx / Apache 自動設定（VI操作不要）
- ドメイン追加・変更時に設定ファイルを **自動生成・書き込み**
- `nginx -s reload` / `systemctl reload apache2` を **自動実行**
- `a2ensite` / `a2enmod` も **自動実行**
- 設定テスト（`nginx -t` / `apachectl configtest`）も **自動実行**
- GUI のトグルスイッチで Apache ↔ Nginx を **ワンクリック切替**

### 🤖 Web サーバー選択アシスタント
- 使用フレームワークをボタンで選択すると最適な Web サーバーを AI が推奨
- WordPress → Apache、Rust/Go/Node.js → Nginx など根拠付きで案内

---

## クイックスタート

```bash
# 1. ビルド
cd f:\aruaru\aruaru-easyweb
cargo build --release

# 2. 設定
cp .env.example .env
# .env を編集して SMTP 設定・管理者メールを入力

# 3. 起動
./target/release/aruaru-easyweb

# 4. ブラウザでアクセス
# http://<VPS_IPアドレス>:8080
```

---

## VPS デプロイ

```bash
# バイナリを VPS にコピー
scp target/release/aruaru-easyweb user@your-vps:/home/aruaru-easyweb/

# systemd サービス登録
sudo cp aruaru-easyweb.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now aruaru-easyweb

# 状態確認
sudo systemctl status aruaru-easyweb
```

---

## ディレクトリ構造

```
aruaru-easyweb/
├── Cargo.toml
├── .env.example                    # 環境変数テンプレート
├── config/default.toml             # アプリ設定
├── aruaru-easyweb.service          # systemd サービス
├── src/
│   ├── main.rs                     # エントリポイント
│   ├── config.rs                   # 設定管理
│   ├── scheduler.rs                # 証明書自動更新スケジューラ
│   ├── auth/                       # Magic Link 認証
│   ├── db/                         # SQLite + マイグレーション
│   ├── domain/                     # ドメイン管理 + SSL
│   ├── webserver/                  # Nginx / Apache 自動制御
│   │   ├── nginx.rs                # 設定生成・書き込み・リロード
│   │   ├── apache.rs               # 設定生成・書き込み・リロード
│   │   └── recommender.rs          # AI 推奨ロジック
│   ├── routes/                     # API エンドポイント
│   └── templates/                  # HTML テンプレート
├── static/
│   ├── css/app.css                 # ダーク UI
│   └── js/app.js                   # トグル・モーダル操作
└── data/
    └── aruaru.db                   # SQLite（自動生成）
```

---

## 技術スタック

| 項目 | 技術 |
|------|------|
| 言語 | Rust 2021 edition |
| Web フレームワーク | Poem 3 |
| DB | SQLite（sqlx 0.7）|
| 証明書 | Let's Encrypt / certbot |
| テンプレート | HTML（静的埋め込み）|
| スケジューラ | tokio-cron-scheduler |
| メール送信 | lettre 0.11 |

---

## ライセンス

aruaru project

---

## VPS 起動手順（VI / .env 操作不要）

```bash
# 1. バイナリを起動するだけ
./aruaru-easyweb

# 2. ブラウザで http://<IPアドレス>:8080 を開く
#    → 自動的に /setup （初回セットアップ画面）が表示される
```

### セットアップ画面でできること

ブラウザのフォームに入力するだけで設定完了：

| 項目 | 説明 |
|------|------|
| アプリの URL | `http://IPアドレス:8080` または `https://your-domain.com` |
| 管理者メール | ログインリンクの受け取り先 |
| SMTP サーバー | `smtp.gmail.com` など |
| SMTP ポート | `587`（Gmail TLS）|
| メールアドレス | 送信元アドレス |
| パスワード | Gmail の場合はアプリパスワード |

設定は DB（SQLite）に保存されます。.env ファイルの編集は不要です。

### 設定の変更

ログイン後、ナビバーの「⚙️ 設定」から いつでも変更できます。
