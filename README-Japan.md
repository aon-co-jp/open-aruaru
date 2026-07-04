# open-aruaru（別名 iLumi プロジェクト）

**「消えない」を、Pure Rustで当たり前にする。**

3DオンラインGAMEの課金アイテムも、金融の残高も、あなたが積み上げたデータも。
open-aruaruは、24時間365日ノンストップで動く"消失しないインターネット"を、
Rustという一つの言語だけで、フロントからDBの奥底まで貫いて実現するプロジェクトです。

> ⚠️ このリポジトリは開発ワークスペース全体（**Private 運用**）です。
> 公開プロダクトは各プロジェクトの独立リポジトリから参照してください。

---

## なぜワクワクするのか

普通、Webサービスを1本作るには「フロントはJS」「バックエンドはGo/Python」「DBはC++製ミドルウェア」…と言語がバラバラで、境界にバグが潜みます。open-aruaruは違います。**画面からストレージの最深部まで、全部Rust。** 型が繋がっているから、書けば書くほど壊れにくくなる。そんな開発体験を目指しています。

そして中核にあるのが **open-runo** ——WunderGraph Cosmo(Go製の名作GraphQLフェデレーション)へのリスペクトを込めつつ、Pure Rust + 独自AI的な予測・選択判断で「そもそもサーバーに聞きに行かない」という発想で高速化と負荷軽減を両立する調停エンジンです。この1本を中心に、アプリケーション層が放射状に広がっています。

---

## エコシステム全体図

```
        open-e-gov      OpenRedmine     OpenWordPress     aruaru-llm
        (電子政府)     (Rust版Redmine) (Rust版WordPress)   (独自LLM)      ← 構想/計画
             │               │               │                │
             └───────────────┴───────┬───────┴────────────────┘
                                     │  GraphQL Federation / VersionlessAPI
                                     ▼
                        ┌─────────────────────────┐
                        │       open-runo         │  ← 中心フレームワーク【公開済み】
                        │  15クレート/176テスト    │     github.com/aon-co-jp/open-runo
                        └───────────┬─────────────┘
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
        aruaru-db             aruaru-ai / -crawler     open-cuda
   (Git-on-SQL 分散DB          (AI 基盤・自動収集)     (GPU 計算 / iLumi)
    pgwire :5433)
```

## プロダクトラインナップ

| プロダクト | ひとこと |
|---|---|
| **open-web-server** | 三層の入口。Apache/Nginxのように他言語アプリも前段に置ける柔軟さと、契約不要のローカル自己学習オートコンフィグを両立 |
| **open-runo** | QUIC+gRPCの二重受け口で三重コミットを裁く、プロジェクトの心臓部 |
| **aruaru-db** | 二重チェックサムで「ネット破損」と「ディスク破損」を切り分ける、追記専用の鉄壁ストレージ |
| **open-client** | Tauri v2製。Electronより軽く速く、予測キャッシュでサーバー往復を間引くネイティブフロント |
| **OpenCosmo** | GraphQLフェデレーションを Pure Rust + Poem で再構築 |
| **OpenLLM(aruaru-llm)** | 独自LLM基盤。open-runoの予測エンジンを本物のモデルへ進化させる次の一手 |
| **OpenDirectX / OpenCuda** | 3DオンラインGAMEのグラフィックス・GPU計算をRustエコシステムへ |
| **Open-eGov / open-e-gov** | 行政サービス(DigitalGovernment)をRust+Poem+OpenRunoで |
| **OpenRedmine / OpenWordPress** | 使い慣れたツールを、同じ三層の安全性の上で動かす |

一つずつはシンプルでも、全部が同じ言語・同じ調停エンジンで繋がった瞬間、掛け算のように可能性が広がります。

## 今すぐ始められる理由

- **迷わない構成** — `open-web-server → open-runo → aruaru-db` という一直線の三層。役割がぶつからない。
- **手を汚さない運用** — Linux設定ファイルは自己学習オートコンフィグが自動生成。AlmaLinux/RHEL/Rocky/Ubuntu/Mint、主要ディストリ全対応。
- **引っ越しも怖くない** — 学習した"賢さ"はrcloneでクラウドへ自動バックアップ。OS再インストールしても復元一発。
- **学習リソースも公式一本道** — [Rust公式ドキュメント](https://doc.rust-lang.org/book/)・[Poem](https://docs.rs/poem/latest/poem/)・[Tauri v2](https://v2.tauri.app/ja/) を読めば、このプロジェクトの隅々まで理解できる設計にしています。

## こんな人にインスピレーションが湧きます

- 「フロントもバックエンドもDBも、1つの言語で書き切りたい」と思ったことがある人
- ゲームの課金アイテムや金融データが「消えない」ことの重みを知っている人
- Go製Cosmoの発想は好きだけど、Rustでもっと速く・軽くできるはずだと感じている人
- 自分のPCで動かしたAIが、次の日も、機種変更しても、賢いままでいてほしい人

このREADMEを読み終えた今この瞬間から、あなたのアイデアは open-web-server / open-runo / aruaru-db の上にそのまま積み上げられます。次に作るのは、あなたの番です。

---

## プロジェクト一覧

### 🏛️ コア基盤

| プロジェクト | 説明 | 状態 |
|--------------|------|------|
| **[open-runo/](open-runo/)** | GraphQL Federation プラットフォーム / Web フレームワーク。Cosmo 有料版機能を OSS 化 + 自己学習 AI キャッシュ・KeyGuardian・DUAL DATABASE 自動修復・お引越し/復活ツールを搭載 | ✅ **v0.1.0 公開** → [aon-co-jp/open-runo](https://github.com/aon-co-jp/open-runo) |
| **[open-web-server/](open-web-server/)** | 3Dオンラインゲームの課金アイテムや金融データを扱う、Rust+Poem製の三層防御Webサーバー | 🔨 開発中 |
| **[aruaru-db/](aruaru-db/)** | Pure Rust の Git-on-SQL 分散データベース。pgwire 互換（:5433）なので PostgreSQL ドライバがそのまま使える。open-runo の DUAL DATABASE の片翼 | 🔨 開発中 |
| **[aruaru-core/](aruaru-core/)** | エコシステム共通のコアライブラリ | 🔨 開発中 |
| **[open-cuda/](open-cuda/)** | GPU 計算クレート（iLumi）。OpenDirectX / OpenCuda 構想の実装 | 📐 設計 |

### 🤖 AI

| プロジェクト | 説明 | 状態 |
|--------------|------|------|
| **[aruaru-ai/](aruaru-ai/)** | ユーザー向け AI 開発アシスタント（aruaru-web に同梱） | 📐 MVP 設計 |
| **[aruaru-crawler/](aruaru-crawler/)** | AI 連携クローラ。言語データの日次自動更新ジョブなど | 🔨 開発中 |
| open-runo 内蔵 AI | 外部 LLM 契約不要の自己学習エンジン（キャッシュ判定・異常検知・自動運用） | ✅ 実装済み |

### 🌐 Web ツール & アプリ

| プロジェクト | 説明 | 状態 |
|--------------|------|------|
| **[aruaru-easyweb/](aruaru-easyweb/)** | Rust 製の簡単 Web サイト構築 | 🔨 開発中 |
| **aruaru-readme-auto-rs**（ルート `src/`） | README.md を FTP アップロードすると全解像度対応（スマホ〜16K）のレスポンシブ HTML を自動生成する Rust + Poem サーバー。詳細: [README-aruaru-readme-auto-rs.md](README-aruaru-readme-auto-rs.md) | ✅ v0.1 |
| **[iZoom/](iZoom/)** | アプリ（apps + backend） | 🔨 開発中 |
| **[rental-sync-hub/](rental-sync-hub/)** | レンタル物件データ同期ハブ | 🔨 開発中 |
| **[frontend/](frontend/)** | 共通フロントエンド | 🔨 開発中 |

### 🔧 開発支援ツール

| プロジェクト | 説明 | 状態 |
|--------------|------|------|
| **[aruaru-github/](aruaru-github/)** | GitHub 連携ツール（Rust） | 🔨 開発中 |
| **[github-easy-push/](github-easy-push/)** | かんたん GitHub push ツール（Node） | ✅ 利用中 |
| **[ftp-auto-sync/](ftp-auto-sync/)** | FTP 自動同期ツール（配布版あり） | ✅ 利用中 |

### 📚 横断ドキュメント

[docs/](docs/) — 製品ビジョン（SPEC_v0.3 second-kusanagi-vision）、システム全体の
DUAL DATABASE 設計、技術スタック選択ガイド（STACK_SELECTOR）ほか。

## 将来プロダクト（open-runo 上に構築）

すべて Rust + Poem + open-runo をバックボーンに開発予定です:

- **open-e-gov** — 電子政府（Open-eGov / DigitalGoverMent）
- **OpenRedmine** — Rust 版 Redmine 互換
- **OpenWordPress** — Rust 版 WordPress 互換（open-runo の AI HTML キャッシュで高速化）
- **aruaru-llm**（OpenLLM） — 独自 LLM
- **OpenDirectX / OpenCuda** — GPU 計算基盤

導入方法は [open-runo/PORTING.md](open-runo/PORTING.md) 1 枚で完結します。

## 技術スタックの原則

- **言語**: Pure Rust（フロントは TypeScript、フレームワーク非依存）
- **Web**: [Poem](https://docs.rs/poem/latest/poem/) + Tokio
- **DB**: DUAL DATABASE（PostgreSQL + aruaru-db）を基本に 9 エンジン対応
- **AI**: 有料 LLM 契約に依存しない自己学習型（使うほど賢くなる）
- **命名**: クレート `open-runo-*` / Rust パス `open_runo_*` / 環境変数 `OPEN_RUNO_*`

## 開発の始め方

```bash
# 中心フレームワークをビルド & テスト
cd open-runo
cargo test --workspace     # 176 テスト

# README 自動変換サーバー（ルートのプロジェクト）
cargo run -- --root /var/www/html --listen 127.0.0.1:7878 --output rs
```

## セキュリティ方針

- 秘密鍵・`.env`・証明書は **絶対にコミットしない**（ルートの `.gitignore` が除外。
  鍵ファイルはこのフォルダの外での保管を推奨）
- このワークスペースリポジトリは **Private** を維持する
- 公開するプロジェクトは独立リポジトリに切り出す（例: open-runo）

## 関連リンク

- Rust Documentation: https://doc.rust-lang.org/stable/ / https://doc.rust-lang.org/book/
- Poem: https://docs.rs/poem/latest/poem/ / https://github.com/poem-web/poem
- Tauri: https://v2.tauri.app/ja/ / https://github.com/tauri-apps/tauri
- WunderGraph Cosmo(着想元): https://wundergraph.com/cosmo

## License

各プロジェクトのライセンスに従います（open-runo は Apache-2.0 OR MIT）。
