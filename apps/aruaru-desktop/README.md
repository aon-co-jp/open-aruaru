# aruaru-desktop

第二の KUSANAGI — Windows デスクトップ版（Tauri 2）

## 機能

| タブ | 内容 |
|------|------|
| ドライブ独立 | DATA / アプリを E: 等に分離、footprint 作成 |
| OS-Recovery | OS 再インストール後の scan / 復活 |
| バックアップ | robocopy ミラー |
| **VPS アップロード** | `deploy/upload-vps.ps1` を実行 |
| プラグイン | aruaru-os-recovery 等 |

## ビルド（インストーラー付き）

```powershell
cd F:\aruaru\apps\aruaru-desktop
.\installer\build-installer.ps1
# 出力: dist\*.exe (NSIS)
```

## OS 再インストール後

1. インストーラーで aruaru-desktop を再インストール（C:）
2. **OS-Recovery** タブ → footprint スキャン → アクセス復活
3. DATA は E:/F: に残ったまま

## 新規 VPS

1. `config/vps.toml` を設定
2. **VPS アップロード** タブ → アップロード + セットアップ
3. ブラウザでドメイン登録

仕様: `docs/SPEC_v0.2-portable-vps-desktop.md`
