# Cursor ドキュメント参照（aruaru）

@Docs 手動登録がうまくいかない場合、**このリポジトリに代替を組み込み済み**です。

---

## 自動セットアップ（推奨）

**Cursor 入れ直し / OS 再インストール後も同じコマンド:**

```powershell
cd F:\aruaru
.\deploy\enable-cursor-setup.ps1
```

またはデスクトップから（ショートカット登録後）:

```powershell
.\deploy\enable-cursor-setup.ps1 -RegisterShortcut   # 初回のみ: デスクトップにショートカット作成
# 以降: デスクトップ「aruaru Cursor Setup」ダブルクリック
# または: deploy\restore-cursor-setup.cmd
```

Docs registry のみ更新する場合:

```powershell
.\deploy\register-cursor-docs.ps1
```

確認:

```powershell
.\deploy\verify-cursor-docs.ps1
```

その後 **Ctrl+Shift+P → Reload Window** で Cursor を再読み込み。

### 前提（再インストール後）

| 必要 | 用途 |
|------|------|
| Cursor | MCP / ワークスペース |
| Node.js 18+ | MCP fetch (`npx @kazuph/mcp-fetch`) |
| `F:\aruaru` | プロジェクト（バックアップから復元） |

復元ログ: `%USERPROFILE%\.cursor\aruaru-setup.json`

---

## 代替の仕組み（3層）

| 層 | 内容 | 使い方 |
|----|------|--------|
| **1. ローカル Docs** | `docs/@docs/*.md`（12件+） | `@docs/@docs/WunderGraph-Cosmo.md` |
| **2. MCP fetch** | `.cursor/mcp.json` | エージェントが公式 URL をライブ取得 |
| **3. Rules** | `.cursor/rules/aruaru-*.mdc` | 常時 / ファイル別の方針 |
| **4. Web 検索** | エージェントの検索 | 公式 Docs に無いとき `site:公式ドメイン` で補完 |

### チャット例

```
@docs/@docs/Poem.md /api/health を拡張して
```

```
@docs/@docs/Tauri.md @docs/@docs/Rust.md aruaru-github プラグインの Tauri 骨組み
```

---

## @Docs 手動登録（任意・併用可）

Cursor には **@Docs をファイルから一括登録する公式 API がありません**（`state.vscdb` への直接書き込みは破損リスクのため非推奨）。

手動で試す場合のみ:

1. **Settings** (`Ctrl + ,`) → 検索 **Docs**
2. **Add new doc** で下表を1件ずつ追加

| @Docs 名 | URL |
|----------|-----|
| Tauri | https://tauri.app/develop/ |
| Rust | https://doc.rust-lang.org/book/ |
| Poem | https://docs.rs/poem/latest/poem/ |
| WunderGraph | https://docs.wundergraph.com/ |
| WunderGraph-Cosmo | https://cosmo-docs.wundergraph.com/ |
| Bootstrap | https://getbootstrap.com/docs/ |
| TypeScript | https://www.typescriptlang.org/docs/ |
| PostgreSQL | https://www.postgresql.org/docs/ |
| SQLx | https://docs.rs/sqlx/latest/sqlx/ |
| VersionLess-API | https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate |

手動登録が失敗しても、**ローカル Docs + MCP fetch で同等の開発が可能**です。

---

## 手動登録が失敗する主な原因

| 原因 | 対処 |
|------|------|
| Settings に Docs が見つからない | Cursor を最新版に更新 |
| URL のインデックスが終わらない | 数分待つ / URL を短くする（docs.rs は crate 直下） |
| @Docs が候補に出ない | 名前のスペル確認 / Reload Window |
| 無料プラン制限 | ローカル `docs/@docs/` を使用（制限なし） |

---

## 登録一覧マシン可読

`.cursor/docs-registry.json` — スクリプト・エージェント用

---

## 関連ファイル

- `.cursor/mcp.json` — fetch MCP
- `.cursor/rules/aruaru-stack.mdc` — スタック全体
- `.cursor/rules/aruaru-wundergraph-cosmo.mdc` — Cosmo + Cursor 管理画面方針
- `.cursor/rules/aruaru-local-docs.mdc` — ローカル Docs 参照
- `docs/STACK_SELECTOR.md` — 言語・FW 比較・引き継ぎ情報
- `.cursor/rules/aruaru-docs-fallback.mdc` — Web 検索フォールバック
- `deploy/register-cursor-docs.ps1` — セットアップスクリプト

---

## フォールバック（公式 Docs が足りないとき）

1. `docs/@docs/<技術>.md` → MCP **fetch** で公式 URL を取得
2. 不足時は **Web 検索**（`site:tauri.app` など公式ドメイン指定）
3. `docs/STACK_SELECTOR.md` で技術文脈を確認

| 手段 | 状態 |
|------|------|
| Cursor @Docs | 手動登録失敗時は未使用 |
| ローカル `docs/@docs/` | 利用可（WunderGraph + Cosmo 含む） |
| MCP fetch | 設定済み（要: Settings で ON） |
| Google / Web 自動検索 MCP | **未設定**（エージェントの Web 検索で代替） |
| Brave Search MCP | 未設定（API キー要） |
