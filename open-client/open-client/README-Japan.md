# open-client

**「消えない」を、Pure Rustで当たり前にする — open-aruaru（別名 iLumi）ネイティブフロント**

Electronではなく **Tauri v2** を選びました。OSネイティブのWebViewを使うためバンドルサイズ・常駐メモリが大幅に小さく、Rustコアを open-web-server / open-runo / aruaru-db と共有できます。さらに独自の**予測キャッシュ判断エンジン**が、サーバーへの往復を賢く間引き、体感速度とサーバー負荷の両方を改善します。

---

## なぜ Tauri なのか

| 観点 | Electron | Tauri |
|---|---|---|
| ランタイム | Chromium 同梱 | OSネイティブWebView |
| バンドルサイズ | 大 (100MB超が一般的) | 小 (数MB〜) |
| 常駐メモリ | Node.js + Chromium 分 | 大幅に少ない |
| アタック面 | Node.js常駐プロセスあり | 最小限のRustコアのみ |
| バックエンド言語 | JS/Node | **Rust**（open-runo等と共通） |

3層(open-web-server / open-runo / aruaru-db)がすべて Rust で書かれているため、フロントエンドも Rust にすることで型定義やチェックサム計算ロジックをそのまま共有でき、実装ミスによる整合性事故を減らせます。

## 予測キャッシュ判断エンジン (predictive.rs)

WunderGraph Cosmo が Go 言語製のGraphQLフェデレーションであるのに対し、open-runo は **Pure Rust + 独自AI的な予測・選択判断**でサーバー負荷を減らす設計を採っています。open-client はその思想をクライアント側まで伸ばしたものです。

- key 毎のアクセス頻度(指数移動平均)を追跡し、「よく読まれる・非金融データ」だけを短TTLでローカルキャッシュ許可。
- **金融データ(残高等)は予測対象外** — 常にサーバーへの多数決読み取り(quorum read)を強制し、キャッシュによる不整合を構造的に排除。
- 予測が外れ続けたキーは自動的に「信頼できない」判定へ降格し、安全側(サーバー確認)に倒れる。
- 将来的に aruaru-llm(OpenLLM)ベースの実モデルへ差し替え可能な `Predictor` トレイト設計。

書き込みは常に open-runo への三重コミット経路を通ります。予測エンジンは**読み取りの前捌きのみ**を担当し、完全性保証を一切バイパスしません。

## 通信経路

```
open-client ──QUIC(udp/4433)──▶ open-runo ──▶ aruaru-db ×3
     │                              (三重コミット確定経路)
     └─(将来) HTTPS ──▶ open-web-server ──▶ 同上
```

GAME/リアルタイム系は open-web-server を経由せず open-runo へ直結し、往復を1段減らしています。

## クイックスタート

```bash
cd src-tauri
set RUNO_QUIC_ADDR=127.0.0.1:4433
cargo tauri dev
```

配布ビルド:

```bash
cargo tauri build
```

## 関連リポジトリ

| リポジトリ | 役割 |
|---|---|
| open-web-server | Layer 1: フロントWEBサーバー |
| open-runo | Layer 2: トランザクション調停 |
| aruaru-db | Layer 3: 三重書き込みストレージ |
| open-client | 統合フロントエンドシェル（本リポジトリ） |

## ライセンス

MIT OR Apache-2.0
