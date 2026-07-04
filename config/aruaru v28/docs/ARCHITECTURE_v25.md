# v25 Architecture

## TypeScript縮小

UIは GraphQL query/mutation を発行するだけの薄い層にします。
登録、クロール、README生成、AIルーティング、ドライブ調査ポリシーはRust側で扱います。

## Rust中心の流れ

```text
UI -> GraphQL -> Poem gateway -> Versionless Function registry -> Rust worker crates
                                  -> aruaru-ai/aruaru-llm
                                  -> README report
                                  -> aruaru-web display
```

## GraphQL Federation / Cosmo

`cosmo/subgraphs/aruaru-platform-v25.graphql` をCosmoへpublishし、Routerから `aruaru-gql-gateway` の `/graphql` にルーティングします。

## 禁止事項

- Tauri を使わない
- REST API を追加しない
- TypeScriptに中核処理を戻さない
