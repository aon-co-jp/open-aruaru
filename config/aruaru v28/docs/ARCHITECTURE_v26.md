# aruaru platform v28 Architecture

## 中核方針

UIは薄く、処理は Rust 側に集約します。

```text
aruaru-web / aruaru-desktop / aruaru-mobile
        │ GraphQL
        ▼
WunderGraph Cosmo Router
        │ Federation
        ▼
Rust + Poem aruaru-gql-gateway
        ├─ aruaru-versionless
        ├─ aruaru-crawler
        ├─ aruaru-drive-indexer
        ├─ aruaru-ai
        └─ aruaru-llm
```

## 機能対応表

| 要件 | v28の実装 |
|---|---|
| 好きなドメイン登録 | `registerDomain` |
| VPS IP / ディレクトリ登録 | `registerServer` / `registerProjectSource` |
| aruaru-desktop / web / mobile連携 | `registerDevice` |
| GitHub URL登録 | `registerProjectSource.githubUrl` |
| ローカルドライブ調査 | `scanDriveNow` + allowlist |
| 24時間5分単位クロール | Gateway起動時 tokio interval 300秒 |
| AI README生成 | `runFullUpdateCycle` / `generateReadmeReport` |
| GitHub用README出力 | `workspace/aruaru（README）/README.md` |
| aruaru.tokyo表示 | `latestReadmeReport` query |
| オフライン閲覧候補 | `registerOfflineSource` |
| GT730軽量化 | `LlmProfile::gt730_safe_default()` |

## セキュリティ方針

ローカルドライブ調査は、ユーザーが明示的に指定したパスだけを allowlist として扱います。全ドライブ自動探索は初期実装では行いません。
