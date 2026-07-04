# aruaru Ver 0.2 — ポータブル + VPS + デスクトップ（第二の KUSANAGI）

> OS 再インストール / 新規 VPS 借り直し後も、**ドメイン登録 → HTTPS 自動化 → 運用**まで最短で復帰する。

---

## 製品構成（2 面）

| 面 | 名称 | 役割 |
|----|------|------|
| **A. Windows デスクトップ** | `aruaru-desktop`（Tauri + インストーラー） | VPS アップロード、DATA 退避、OS 再インストール後の復活 |
| **B. VPS サーバー** | `aruaru serve`（Linux + Poem + PostgreSQL） | Web 版 KUSANAGI — ドメイン / HTTPS / easyweb |

```
┌─────────────────────────┐         upload          ┌──────────────────────────┐
│  Windows デスクトップ      │  deploy/upload-vps.ps1  │  VPS (ConoHa 等)          │
│  aruaru-desktop.exe     │ ──────────────────────► │  aruaru serve :3000       │
│  インストーラー (NSIS)   │                         │  PostgreSQL + easyweb     │
│  DATA on E:/F:          │                         │  DATA on /var/lib/aruaru  │
└─────────────────────────┘                         └──────────────────────────┘
         │ footprint                                              │ platform_domains
         │ .aruaru-footprint.json                               │ https_monitor
         ▼ OS 再インストール後 scan/restore                      ▼ 自動 https://
```

---

## コア機能（ユーザー要求 ↔ 実装状況）

| 機能 | 状態 | パス / 備考 |
|------|------|-------------|
| ドメイン登録（本体 + サイト） | ✅ MVP | `platform_domain.rs`, `/settings/domain`, easyweb |
| DNS ガイド + レコード CRUD | ✅ MVP | `dns.rs`, `aruaru-dns` Tauri |
| HTTPS 自動監視 | ✅ 監視 | `https_monitor.rs` — 5 分間隔 probe |
| HTTPS 自動付与（certbot） | 🔄 v0.2 | 現状 stub → `data/https-queue/` |
| vhost / redirect / proxy | 🔄 v0.2 | manifest のみ |
| VPS 一括アップロード | 🔄 着手 | `deploy/upload-vps.ps1` |
| Windows インストーラー | 🔄 着手 | `apps/aruaru-desktop/installer/` |
| DATA を C: 以外に退避 | ✅ MVP | `aruaru-desktop` ドライブ独立 |
| OS 再インストール後復活 | ✅ MVP | footprint scan + restore |
| VPS Linux DATA 退避 | 🔄 v0.2 | `deploy/linux-data-portable.sh` |
| デスクトップから VPS UPLOAD | 🔄 着手 | desktop VPS タブ |

---

## DATA ポータビリティ設計

### Windows

```
C:\Program Files\aruaru\          ← アプリ（再インストール可）
%LOCALAPPDATA%\aruaru\desktop.toml ← ポインタ（復元で再生成）

E:\aruaru\                         ← root_dir（ソース / 設定）
E:\aruaru-data\                   ← data_dir
  .aruaru-footprint.json          ← 復元マーカー
  config.toml, recovery/, plugins/
```

**フロー:** OS 再インストール → インストーラーで desktop のみ → footprint スキャン → アクセス復活 → サーバ起動（DATA そのまま）

### Linux (VPS)

```
/opt/aruaru/                      ← バイナリ + リポジトリ（再デプロイ可）
/var/lib/aruaru/                  ← data_dir（推奨）
  .aruaru-footprint.json
  PostgreSQL データは別（pg_dump バックアップ）
```

**フロー:** OS 再インストール → `upload-vps.ps1` → footprint 確認 → `vps-full-setup.sh`

---

## デスクトップ版 — 画面構成

| タブ | 内容 |
|------|------|
| ドライブ独立 | DATA / root を E: 等に設定、footprint 作成 |
| OS-Recovery | 再インストール後スキャン・復活 |
| バックアップ・同期 | robocopy |
| **VPS アップロード** | upload-vps.ps1 実行、状態表示 |
| プラグイン | easyweb / os-recovery 等 |

---

## VPS 版 — 初回セットアップ（目標 10 分）

1. Windows: `aruaru-desktop` または `deploy/upload-vps.ps1`
2. VPS: `vps-full-setup.sh`（PostgreSQL + build + systemd）
3. ブラウザ: `http://VPS_IP:3000/register`
4. `/settings/domain` でドメイン登録
5. レジストラで A レコード → `/api/dns/guide` 参照
6. `https_monitor` が HTTPS 化を検知 / v0.2 で certbot 自動

---

## 開発フェーズ

| Phase | 内容 |
|-------|------|
| **0.2a** | upload-vps.ps1 + desktop VPS タブ + installer 骨組み |
| **0.2b** | certbot 本実装、vhost テンプレート |
| **0.2c** | Linux DATA footprint + VPS 復元スクリプト |
| **0.2d** | ワンクリック「新 VPS 借り直し」ウィザード |

---

## Cursor ルール

`.cursor/rules/aruaru-portable-vps-desktop.mdc` — 本仕様を常時参照

## 関連

- `docs/SPEC_v0.1.md` — ベース仕様
- `apps/aruaru-desktop/` — Tauri デスクトップ
- `deploy/vps-full-setup.sh` — VPS セットアップ
- `deploy/upload-vps.ps1` — Windows → VPS アップロード
