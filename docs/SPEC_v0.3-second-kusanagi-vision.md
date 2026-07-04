# aruaru Ver 0.3+ — Second KUSANAGI ビジョン（2026-06-24）

> **コンセプト:** Windows 感覚で Linux サーバーを使える AI アシスタント  
> **タグライン:** 初心者からプロまで、AI と一緒に開発・学習・運用できる統合プラットフォーム

---

## 製品構成（2 面・同一アカウント）

| 面 | 名称 | 主な利用シーン |
|----|------|----------------|
| **デスクトップ** | `aruaru-desktop`（Tauri） | SSH 鍵・接続ウィザード、VPS アップロード、DATA 退避、OS 復元、IDE |
| **Web** | `aruaru-web`（`aruaru serve` + Poem） | サーバーダッシュボード、ドメイン/HTTPS、機能一覧、AI チャット、共同編集 |

両者は **設定・鍵・プロジェクト・プラグイン・AI プロファイル** を共有する。

---

## 3 層アーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│  aruaru-desktop / aruaru-web（UI）                       │
├─────────────────────────────────────────────────────────┤
│  1. AI Core          モデル選択・エージェント・安全実行   │
│  2. Development Core VS Code 互換・Git・Live Share・CI  │
│  3. Platform Core    SSH・VPS・Docker・HTTPS・監視・BK   │
├─────────────────────────────────────────────────────────┤
│  Rust + Poem + PostgreSQL/CockroachDB（DUAL）           │
│  VersionLess API + WunderGraph Cosmo（OSS）             │
│  プラグイン（easyweb / recovery / dns / ai / character） │
└─────────────────────────────────────────────────────────┘
```

### AI 安全実行エンジン（中核）

AI が生成した操作は **即実行しない**:

1. コマンド生成
2. 危険度判定（`rm -rf /` 等は拒否）
3. 影響範囲の説明（初心者向け日本語）
4. 必要ならスナップショット / バックアップ
5. ユーザー承認（「はい」のみ）
6. 実行 → ログ保存 → 失敗時ロールバック案内

---

## 初心者向け UI イメージ（aruaru-web ダッシュボード目標）

```
━━━━━━━━━━━━━━━━━━━━━━━
 AI Server Manager
━━━━━━━━━━━━━━━━━━━━━━━
 サーバー状態 🟢正常
 CPU 12%  メモリ 28%  ディスク 41%
 SSL 正常  ドメイン 正常  バックアップ 昨日完了
━━━━━━━━━━━━━━━━━━━━━━━
 ① Webサイト作成  ② ドメイン追加  ③ SSL追加
 ④ データベース   ⑤ AIに相談
━━━━━━━━━━━━━━━━━━━━━━━
```

**原則:** コマンドを覚えさせない。ボタン・メニュー・AI チャットのみ。

---

## 開発フェーズ（Phase 1〜10）

| Phase | 名称 | 目的 | aruaru 既存との関係 |
|-------|------|------|---------------------|
| **1** | AI 接続アシスタント | 5 分で VPS 接続 | `config/vps.toml`, `check_vps_connection`, `upload-vps.ps1` → **接続ウィザード + AI 診断** |
| **2** | AI サーバー初期設定 | 10 分で初期構築 | `vps-full-setup.sh` → UFW / Fail2Ban / 進捗 UI |
| **3** | HTTPS 完全自動 | ワンクリック HTTPS | `https_cert.rs`, `vhost.rs`, 機能一覧 `https.certbot` ✅ 着手済 |
| **4** | Docker Marketplace | GUI でコンテナ | 新プラグイン `aruaru-docker` |
| **5** | AI 開発環境 | スタック選択構築 | Cosmo + Poem + aruaru-db テンプレ |
| **6** | AI 運用 | 自然言語で障害解析 | 監視プラグイン + journalctl 解析エージェント |
| **7** | AI 秘書 | 「ブログ公開したい」→ 一式 | easyweb + WordPress プラグイン |
| **8** | 第二の KUSANAGI 統合 | 1 画面で全部 | desktop + web 機能統合 |
| **9** | AI 先生 | 図解・演習・理解度 | 学習プラグイン + aruaru-llm |
| **10** | AI エージェント | 要件→ GitHub→デプロイ | Cursor 連携・CI/CD プラグイン |

**MVP（Phase 1〜3）だけでも「Linux 初心者が VPS を諦めない」価値を提供できる。**

---

## Phase 1 詳細 — SSH を意識させない

### 接続ウィザード

1. プロバイダ選択（ConoHa / さくら / AWS / Azure / GCP / Oracle / その他）
2. IP・ユーザー名（既定 `root`）
3. 秘密鍵ファイル選択（または新規 Ed25519 生成）
4. 「接続」→ 自動チェック

### 自動診断項目

- IP / ポート 22
- 鍵の存在・パーミッション
- `known_hosts`
- ファイアウォール
- 公開鍵と VPS `authorized_keys` の一致

失敗時は **原因 + 修正手順** を日本語で表示（AI 診断オプション）。

### 鍵管理（desktop）

```
接続先一覧
  ConoHa    🟢  ConoHa_Main.pem
  AWS       🟢  aws-prod.pem
```

初回: 鍵なし → Ed25519 生成 → 安全保存 → 公開鍵コピー → ConoHa 貼り付け案内  
将来: プロバイダ API で公開鍵登録自動化

---

## AI エージェント

### モデル選択（プラグイン）

- ChatGPT / Claude Opus・Sonnet / Gemini / Grok / DeepSeek / Qwen / Llama / Mistral
- OpenRouter / Ollama（ローカル）
- **aruaru-llm**（研究開発中・オリジナル）

用途別プリセット例: 設計=Opus, 実装=GPT, レビュー=Gemini

### AI 先生

- レベル: 初心者 / 中級 / 上級 / プロ
- 科目: Python, Rust, TypeScript, Linux, Docker, Poem, Tauri, Cosmo, SQL …
- **キャラクターはプラグイン**（Character Marketplace）
  - オリジナルデザインのみ（実在人物・既存作品の模倣は不可）
  - アニメ風 / リアル風切替、音声 ON/OFF・音量・速度
- 資格・学習: 英検 / TOEIC / TOEFL 模擬試験、合格証 PDF（将来）

### エディター

- VS Code 互換 UI（Explorer / Terminal / Git / Debug / AI Chat）
- Live Share 連携（人間の先生・友達と共同編集）

---

## 技術スタック（統一）

| 層 | 採用 |
|----|------|
| バックエンド | Rust + Poem |
| デスクトップ | Tauri |
| フロント | TypeScript + Vite |
| API | WunderGraph **Cosmo**（完全 OSS） |
| DB | PostgreSQL + CockroachDB（DUAL / aruaru-db） |
| API 進化 | VersionLess API（ハイブリッドバージョン管理） |
| インフラ | Nginx / certbot / Docker（プラグイン） |

---

## プラグイン体系（拡張）

| カテゴリ | 例 |
|----------|-----|
| Web サーバー | Nginx, Apache, Caddy |
| DB | PostgreSQL, CockroachDB, MySQL |
| Docker / K8s | Marketplace |
| SSL / セキュリティ | Let's Encrypt, Fail2Ban |
| 監視 / ログ | CPU, journalctl, Nginx 解析 |
| AI | モデルプロバイダ、aruaru-llm |
| Character | 先生キャラ（オリジナル） |
| デプロイ / バックアップ | GitHub, スナップショット |
| Framework | Poem, Tauri, Vue, React … |

既存: `aruaru-easyweb`, `aruaru-os-recovery`, `aruaru-dns`

---

## 現状（2026-06-24 時点）

| 領域 | 状態 |
|------|------|
| VPS アップロード | `upload-vps.ps1`, desktop VPS タブ |
| VPS 接続確認 | `check_vps_connection`, フッター表示 |
| HTTPS / vhost | certbot + 機能一覧 |
| ドメイン / DNS | platform_domains, dns ガイド |
| 機能一覧（選択実行） | web + desktop |
| Windows インストーラー | NSIS + MSI |
| DATA ポータビリティ | footprint, OS-Recovery |
| AI 接続ウィザード | **未着手** ← Phase 1 次 |
| AI 安全実行エンジン | **未着手** |
| サーバー監視ダッシュボード | **未着手** ← Phase 2/6 |
| AI チャット / 先生 | **未着手** ← Phase 9 |

---

## Cursor 開発の優先順位（提案）

1. **Phase 1a** — 接続ウィザード UI（desktop + web）、鍵生成・診断
2. **Phase 1b** — AI 接続エラー診断（プロンプト + ルール）
3. **Phase 2** — 初期設定進捗 UI + `vps-full-setup.sh` 拡張
4. **Phase 3 仕上げ** — ダッシュボード「SSL追加」ワンクリック
5. **AI Core 骨組み** — モデル選択 + 安全実行エンジン（承認フロー）

---

## 関連ドキュメント

- `docs/SPEC_v0.2-portable-vps-desktop.md` — ポータブル + VPS 基盤
- `.cursor/rules/aruaru-portable-vps-desktop.mdc`
- `.cursor/rules/aruaru-wundergraph-cosmo.mdc`
- `docs/@docs/WunderGraph-Cosmo.md`
