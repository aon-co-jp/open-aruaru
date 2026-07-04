# aruaru-github

**Windows フォルダをマウスで指定 → GitHub に Push**

Rust + egui 製デスクトップアプリ。インストーラー付き。

## 機能

- 📁 **フォルダ参照** — エクスプローラー風ダイアログでローカルディレクトリを指定
- 🚀 **ワンクリック Push** — `git init` / `add` / `commit` / `push` を自動実行
- 🔗 **GitHub URL** — `https://github.com/user/repo.git` を指定
- 📋 **ログ表示** — 実行内容を画面に表示

## 必要環境

- **Windows 10/11**
- **[Git for Windows](https://git-scm.com/download/win)** — PATH に `git` が通っていること
- **GitHub** — 空のリポジトリを事前作成、または push 権限
- **認証** — Git Credential Manager / PAT / SSH（初回 push 時に Windows が促す）

## 開発・実行

```powershell
cd F:\aruaru\aruaru-github
cargo run
```

## インストーラー作成

```powershell
cd F:\aruaru\aruaru-github
.\installer\build-installer.ps1
```

**exe だけ欲しい場合:**

```powershell
.\installer\build-installer.ps1 -ExeOnly
```

### CRC エラー（巡回冗長検査 / os error 23）が出た場合

E: が **exFAT** の場合、ディスク不良で `target\` への書き込みが失敗することがあります。  
スクリプトは CRC を検出すると **C: 上の `%LOCALAPPDATA%\aruaru-github-target`** に自動切替します。

手動で C: ビルドする場合:

```powershell
.\installer\build-installer.ps1 -SafeTarget -ExeOnly
```

根本対処:

1. **管理者 PowerShell** で `chkdsk E: /F` を実行（exFAT 修復）
2. ウイルス対策の **リアルタイム保護を一時 OFF** して再ビルド
3. 出力は `dist\aruaru-github.exe` を直接実行

| 出力 | 内容 |
|------|------|
| `dist\aruaru-github.exe` | 単体 exe |
| `dist\aruaru-github-setup-0.1.0.exe` | Inno Setup インストーラー（要 Inno Setup 6） |
| `dist\aruaru-github-portable.zip` | Inno 無し時の ZIP |

Inno Setup: https://jrsoftware.org/isdl.php

## 使い方

1. **aruaru-github** を起動
2. **📁 参照…** で Push したいフォルダを選択（例: `F:\aruaru`）
3. **GitHub URL** に `https://github.com/あなた/リポジトリ.git` を入力
4. **🚀 GitHub に Push** をクリック
5. 初回は GitHub ログイン / PAT 入力

## PAT（Personal Access Token）

https://github.com/settings/tokens — `repo` スコープを付与

## ライセンス

MIT — aruaru プロジェクト
