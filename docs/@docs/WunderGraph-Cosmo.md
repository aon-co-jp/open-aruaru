# WunderGraph Cosmo（aruaru GraphQL フェデレーション基盤）

> 公式: https://cosmo-docs.wundergraph.com/  
> GitHub: https://github.com/wundergraph/cosmo  
> LLM 索引: https://cosmo-docs.wundergraph.com/llms.txt

## なぜ Cosmo か

- **完全オープンソース** — 自前 VPS・オンプレ・フォーク可能
- GraphQL **Federation v1/v2** + モノリシック GraphQL API
- Router / Control Plane / Studio / CLI の **フルスタック**
- 商用 WunderGraph Cloud の一部機能は OSS で簡素 → **不足は aruaru + Cursor で管理画面等を補完**

## aruaru での使い方

| コンポーネント | 用途 |
|----------------|------|
| **Cosmo Router** | 本番 GraphQL ルーティング |
| **Control Plane** | スキーマ・サブグラフ・composition checks |
| **Cosmo Studio** | OSS 管理 Web UI |
| **CLI `wgc`** | スキーマ push、検証、運用自動化 |
| **aruaru カスタム UI** | Studio で足りない画面 → `frontend/`（TS + Bootstrap、Cursor で開発） |

## VersionLess + DB ハイブリッド

- **DB 層**: `src/storage/` + `migrations/`（論理バージョン）— Cosmo と独立に維持
- **GraphQL 層**: Cosmo でスキーマ進化・フェデレーション
- **HTTP 層**: Poem `/api/*` + VersionLess フィールド追加
- 詳細: `docs/versionless-api.md`, `docs/@docs/VersionLess-API.md`

## 公式リンク

- はじめに: https://cosmo-docs.wundergraph.com/
- Router: https://cosmo-docs.wundergraph.com/router/introduction
- Control Plane: https://cosmo-docs.wundergraph.com/control-plane/introduction
- Studio: https://cosmo-docs.wundergraph.com/studio/introduction
- CLI (`wgc`): https://cosmo-docs.wundergraph.com/cli/introduction
- Federation: https://cosmo-docs.wundergraph.com/federation/introduction
- セルフホスト: https://cosmo-docs.wundergraph.com/self-hosted/introduction

## WunderGraph SDK との関係

- **Cosmo** = フェデレーション・ルーティング・運用基盤（OSS）
- **WunderGraph SDK** = Operations / TS クライアント（`wundergraph/`、`docs/@docs/WunderGraph.md`）
- aruaru では **両方併用**し、Cursor で管理画面・メンテを継続

## Cursor での参照

- ローカル: `@docs/@docs/WunderGraph-Cosmo.md`
- ルール: `.cursor/rules/aruaru-wundergraph-cosmo.mdc`
- @Docs 手動登録名: **WunderGraph-Cosmo** → https://cosmo-docs.wundergraph.com/
