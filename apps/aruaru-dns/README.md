# aruaru-dns — Tauri デスクトップ

DNS レコード管理。PostgreSQL（正）+ aruaru-db（`F:\aruaru\aruaru-db`）ダブル DB。

## 起動

```powershell
cd F:\aruaru\apps\aruaru-dns
npm install
npm run tauri dev
```

## ビルド（インストーラー）

```powershell
npm run tauri build
```

## 前提

1. PostgreSQL（aruaru 本番 DB）が起動していること
2. 任意: aruaru-db サーバーをポート 5433 で起動

## 設定ファイル

`%APPDATA%\aruaru-dns\config.toml` — 未作成時は `src-tauri/config/default.toml` を使用
