# open-web-server

**「消えない」を、Pure Rustで当たり前にする — open-aruaru（別名 iLumi）Layer 1**

課金アイテムも、残高も、あなたが積み上げたデータも。open-web-serverは、その入口に立つ薄くて速い門番です。裏側の複雑さは全部 open-runo に任せ、自分は「受け取って、渡して、結果を返す」ことだけに徹する——このシンプルさが、24時間365日ノンストップという要求に応えるための最短ルートだと私たちは考えています。

Pure Rust ＋ [Poem](https://docs.rs/poem/latest/poem/) 製。書けば書くほど型で守られる開発体験を、フロントのこの1枚から味わえます。

---

## この製品の特長

- **三層アーキテクチャの入口** — 自分ではデータを持たず、open-runo(Layer 2)へ委譲するだけの薄いフロント。障害の複雑さを1箇所(Layer 2)に閉じ込める設計。
- **正式契約(proto)で疎結合** — open-runo との通信は `proto/runo.proto` (gRPC) で厳密に型付け。Layer 2/3 の内部実装が変わっても Layer 1 は再ビルド不要。
- **エンドツーエンド整合性** — 書き込み時に BLAKE3 チェックサムをここで計算し、Layer 3(aruaru-db)まで同じ値を検証に使う。経路上の破損を検出可能。
- **優先度指定** — リクエストに `finance: true` を付けるだけで `PRIORITY_FINANCE`(レコード毎fsync・3/3コミット必須)へ切り替え。ゲームアイテムと金融データを同一APIで安全に使い分け。
- **非到達時も握りつぶさない** — open-runo に届かない場合は `committed_layers: 0` を明示的に返し、呼び出し元にリトライを促す。

## 仕様

| 項目 | 内容 |
|---|---|
| 言語/フレームワーク | Rust + Poem |
| リッスンポート | 8080 (HTTP) |
| Layer 2 接続方式 | gRPC (`RUNO_GRPC_ADDR` 環境変数、既定 `http://127.0.0.1:50051`) |
| 整合性検証 | BLAKE3 (32byte) |
| 対応優先度 | PRIORITY_GAME / PRIORITY_FINANCE |

## API

```bash
# 書き込み (三重コミットへ委譲)
curl -X POST http://localhost:8080/v1/kv \
  -H "Content-Type: application/json" \
  -d '{"key":"user:1234:wallet","value":{"gold":50000},"finance":true}'

# 読み取り (多数決)
curl http://localhost:8080/v1/kv/user:1234:wallet
```

レスポンス例:

```json
{ "txn_id": "01906e2a-...", "committed_layers": 3, "checksum": "af1349b9...", "status": "committed" }
```

`committed_layers: 3` を確認してから課金処理を完了してください。`2` は quorum-safe (安全)、`0` は open-runo 未到達を意味します。

## Apache/NginxのようなWEBサーバーとの違いと相性

Apache/Nginx は「HTTPを受けて別プロセス(PHP-FPM/Node等)へ橋渡しするだけ」の言語非依存な前段サーバーです。open-web-server は本来、Poem(Rust)でロジックそのものを1プロセス内実行する設計のため、**素の状態では他言語アプリとの相性はよくありません**。これを補うため v0.3 で3つのルーティングモードを追加しました。

| モード | 用途 | Apache/Nginx相当 |
|---|---|---|
| `Native` | open-e-gov/OpenRedmine/OpenWordPress等、Rust+Poem+OpenRuno資産 | mod_php直組込み相当(最速) |
| `Proxy` | 既存のPHP/Node/Python資産を前段に置く | mod_proxy / proxy_pass |
| `Static` | SPA・静的アセット配信 | DocumentRoot |

`routes.toml` で切り替えます(後述の自動生成対象)。

## 契約不要のローカル自己学習オートコンフィグ (autoconf)

Linuxサーバー用の設定ファイルを手で書かなくて済むよう、外部AI API等の**契約や通信を一切行わない**ローカル自己学習ループを搭載しました。実体は誇張せず言うと「自リクエストの統計(EMA/簡易p99)を `data/tuning.json` に蓄積し、閾値ルールで再調整する適応コントローラ」です。100リクエスト毎に:

- レイテンシが高ければ `worker_threads` を増加(輻輳対策)
- 十分軽ければ徐々に減らしCPU/メモリを節約(**サーバー負荷軽減**)
- p99が高ければ `keepalive_secs` を短縮し詰まった接続を早期解放

の3点を自動調整し、`data/generated/open-web-server.service`(systemd unit)と `data/generated/routes.toml` を自動で書き出します。前回までの学習結果は `tuning.json` から引き継がれ、再起動のたびに賢くなります(=成長型)。生成物は `/etc/systemd/system/` 等へコピーするだけで運用に反映できます。

## 対応Linuxディストリビューション

| ディストリビューション系統 | パッケージ管理 | ファイアウォール | 備考 |
|---|---|---|---|
| Ubuntu / Debian / Linux Mint | apt | ufw | 標準構成でそのまま動作 |
| RHEL / AlmaLinux / Rocky Linux | dnf (rcloneはEPEL経由 or 公式インストーラ) | firewalld | SELinux有効時は `bin_t` コンテキスト付与が必要 |

`scripts/install-deps.sh` が `/etc/os-release` を見て自動判別し、rcloneの導入・ポート開放(8080/tcp, 50051/tcp, 4433/udp)・SELinuxコンテキスト付与までまとめて行います。

```bash
bash scripts/install-deps.sh
```

## OS再インストール・機種変更をまたいだ「賢さ」の引き継ぎ

Windows/Linux/Mac向けにGoogleドライブ互換クライアントを自作するのは本プロジェクトの範囲を超えるため、**実績のある rclone に同期そのものを任せ、整合性検証だけ aruaru 流儀(BLAKE3)で自前保証する**、という現実的な構成にしています。

- autoconf は学習結果を `data/aruaru-brain.json` に BLAKE3 チェックサム付きで書き出します(100リクエスト毎)。
- 起動時、このファイルが存在すればチェックサムを検証したうえで学習状態を復元します。**検証に失敗したファイル(同期中に壊れた等)は安全側に倒し、新規学習として扱う**ため、誤った状態を引き継ぐ事故を防ぎます。
- `scripts/sync-brain.sh`(Linux/Mac)・`scripts/sync-brain.ps1`(Windows)で `rclone` 経由でGoogleドライブ等へpush/pullします。crontab/タスクスケジューラに登録すれば全自動です。
- **RHEL系(AlmaLinux/RHEL/Rocky Linux)ではcronよりsystemdタイマーが標準的**なため、`scripts/systemd/aruaru-brain-sync.service` + `.timer` も用意しています。`/etc/systemd/system/` に配置して `systemctl enable --now aruaru-brain-sync.timer` するだけで5分毎の自動push が有効になります。

```bash
# 初回のみ (Google認証はここで1回だけ)
rclone config   # リモート名は "gdrive" にする

# 通常運用: 5分毎に自動バックアップ
*/5 * * * * bash /path/to/open-web-server/scripts/sync-brain.sh push

# OS再インストール後・新PCでの復元
bash scripts/sync-brain.sh pull
cargo run --release   # 起動時に aruaru-brain.json を自動検出して学習状態を復元
```

Ubuntu/Linux Mintでは「オンラインアカウント」設定からGoogleアカウントを追加すればNautilus/Nemoからも同じドライブを参照できるので、`aruaru-brain.json` を目視確認したい場合はそちらが手軽です。ファイルの自動同期・復元自体は上記スクリプトが担います。

## 3リポジトリの相性を保つために

- `proto/runo.proto` は open-runo リポジトリと**同一ファイル**を配置しています。変更時は `scripts/check-proto-sync.sh` で両者のハッシュ一致を確認してください。
- Layer 1 はレプリカ数や優先度レーンの実装詳細を一切知りません。Layer 2 側の変更(例: レプリカ4本化)は Layer 1 に影響しません。

## クイックスタート

```bash
# 先に open-runo を起動しておくこと (既定 tcp/50051)
set RUNO_GRPC_ADDR=http://127.0.0.1:50051
cargo run --release
```

## 関連リポジトリ

| リポジトリ | 役割 |
|---|---|
| open-web-server | Layer 1: フロントWEBサーバー（本リポジトリ） |
| open-runo | Layer 2: トランザクション調停 (QUIC+gRPC 二重受け口) |
| aruaru-db | Layer 3: ARU3三重書き込みストレージ |
| open-client | Tauri v2 製ネイティブフロントエンド |
| OpenCosmo | GraphQLフェデレーション (Pure Rust + Poem) |

## GitHubへのpush

`f:\open-aruaru` 直下(ドライブ直結の作業フォルダ)から、5リポジトリ分を一括で `git init`→`commit`→`push` できます。

```bash
export GITHUB_USER=your-account
bash open-web-server/scripts/push-all.sh
# Windows PowerShellの場合:
# .\open-web-server\scripts\push-all.ps1 -GitHubUser your-account
```

事前にGitHub側で同名リポジトリ(`open-web-server`等)を作成し、SSH鍵かGitHub CLIでの認証を済ませておいてください。`data/`(学習ファイル・DBセグメント等の実データ)は自動で`.gitignore`に追加され、誤push を防ぎます。

## ライセンス

MIT OR Apache-2.0

---

*このREADMEを読み終えたら、あとは `cargo run` するだけ。open-runo・aruaru-dbと繋いだ瞬間、あなたのアイデアはもう「消えない」土台の上にあります。*
