# aruaru platform v28

`aruaru-platform-functional-core-v26` を、さらに実装寄りから実装段階へ進めた版です。

## 方針

- REST / REST API は使わない
- Tauri は使わない
- TypeScriptは薄いUIに限定
- 中核は Rust + Poem + GraphQL Federation / WunderGraph Cosmo + Versionless Function
- GT730 PCではローカルGPU LLMを要求せず、ChatGPT / Claude Opus / aruaru-llm / VPS worker を選択

## 中核実装

- `crates/aruaru-gql-gateway`
  - Poem + async-graphql gateway
  - `/graphql` のみ
  - Cosmo Router / Federation前提
- `crates/aruaru-core`
  - platform state
  - domain/server/project/device/offline source model
  - validation
- `crates/aruaru-crawler`
  - GitHub URL / ローカル / VPS情報を5分単位で収集
- `crates/aruaru-drive-indexer`
  - 許可済みローカルドライブのみ調査
- `crates/aruaru-ai`
  - `workspace/aruaru（README）/README.md` 生成
- `crates/aruaru-offline-cache`
  - オンライン時の閲覧キャッシュ保存
  - 3日超の通常キャッシュ自動削除
  - `audiocafe.tokyo` 由来DATAは保持

## オフラインキャッシュ方針

オンライン時に、登録されたニュース・動画ページ・audiocafe候補などをローカル閲覧用HTMLとして保存します。

- 通常DATA: 再取得/整理時に3日以上前のものを削除
- `audiocafe.tokyo` / `*.audiocafe.tokyo`: 自動削除対象外
- YouTube: 動画ファイルそのものではなく、閲覧用ページ/リンク/メタ情報相当を保存

## 起動イメージ

```bash
export ARUARU_WORKSPACE=./workspace
export ARUARU_BIND=127.0.0.1:8787
cargo run -p aruaru-gql-gateway
```

UI:

```bash
cd apps/aruaru-web
npm install
npm run dev
```

## 確認

```bash
./scripts/verify-v28.sh
cargo check --workspace
cargo test --workspace
```

このChatGPT実行コンテナには `cargo/rustc` が無いため、実コンパイルはローカルPCで実行してください。
