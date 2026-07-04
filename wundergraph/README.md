# WunderGraph — aruaru Ver 0.1

aruaru は **VersionLess API**（WunderGraph 思想）と **WunderGraph Cosmo**（GraphQL フェデレーション OSS）を採用する。

## 目標

- **Cosmo**: Router / Control Plane / Studio / `wgc` CLI
- **WunderGraph SDK**: PostgreSQL introspect → Operations → TS クライアント
- Cosmo Studio で足りない管理画面 → **Cursor + TypeScript + Bootstrap** で aruaru 側に補完

## 参照

- Cosmo: `docs/@docs/WunderGraph-Cosmo.md`
- SDK: `docs/@docs/WunderGraph.md`
- ルール: `.cursor/rules/aruaru-wundergraph-cosmo.mdc`
- VersionLess: `docs/versionless-api.md`

## 構成（予定）

```
wundergraph/          # WunderGraph SDK 設定
cosmo/                # Cosmo Router / Control Plane 設定（今後）
frontend/             # TS + Bootstrap + カスタム管理 UI
```

v0.1d フェーズで初期化する。
