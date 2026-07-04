# Cursor への命令文 — VPS 状態を自動把握する仕組み

> このファイルをコピーして Cursor に貼り付けると、  
> **SSH 鍵・接続・リモート診断を Cursor / エージェントが自動で扱える** 仕組みを作れます。

---

## いちばん使いやすい命令文（日本語・コピペ用）

```
私のプロジェクトに、Cursor から VPS 状態を自動把握できる仕組みを追加してください。

## 目的
- 手動で長い ssh コマンドを打たなくてよい
- Cursor エージェントがスクリプト 1 本で VPS + アプリの状態を JSON で取得できる
- Windows PowerShell の SSH 鍵パス問題（C: コロン、引用符）をスクリプト側で吸収する

## VPS 情報
- ホスト: 160.251.237.162
- ユーザー: root
- SSH 鍵: %USERPROFILE%\.ssh\conoha-vps.pem
- アプリポート: 3000
- リモートプロジェクト: /root/aruaru
- ヘルス API: http://127.0.0.1:3000/api/health

## 作ってほしいもの

1. **config/vps.toml** — ホスト・ユーザー・鍵・ポート（秘密情報は含めない）

2. **deploy/cursor-vps.ps1**（PowerShell 専用）— サブコマンド:
   - `status`  … 人間向け表示（SSH / ポート / systemd / health）
   - `check`   … JSON 出力 + exit code（エージェント / CI 用）
   - `connect` … 対話 SSH ログイン
   - `fix-ssh` … ~/.ssh/config 修復

3. **.vscode/tasks.json** — タスク:
   - VPS: 状態確認
   - VPS: SSH 接続
   - VPS: SSH 設定修復

4. **.cursor/rules/vps.mdc** — エージェント向けルール:
   - 手動 `ssh conoha` や `-i C:\...`（引用符なし）は使わない
   - VPS 確認は必ず `.\deploy\cursor-vps.ps1 check` を使う

5. **README** に使い方を追記

## 技術要件
- SSH は鍵ディレクトリに cd するか、`-i "C:/Users/.../key.pem"` のように引用符付きフルパス
- `check` の exit code: 0=全部OK, 2=SSH失敗, 3=ポート未待受, 4=health失敗
- リモート診断: ss/curl/systemctl を SSH 経由で実行
- bash ターミナルではなく **PowerShell 専用** と明記

## 完成後
- `.\deploy\cursor-vps.ps1 status` が動くことを確認
- 使い方を 3 行で教えてください
```

---

## English version (for worldwide sharing)

```
Add a Cursor-friendly VPS status automation system to my project.

## Goal
- No manual long ssh commands
- Cursor agent can get VPS + app status as JSON via one script
- Handle Windows PowerShell SSH quirks (C: drive colon, quoting) inside the script

## VPS
- Host: YOUR_VPS_IP
- User: root
- SSH key: %USERPROFILE%\.ssh\your-key.pem
- App port: 3000
- Remote project dir: /root/your-app
- Health endpoint: http://127.0.0.1:3000/api/health

## Deliverables
1. config/vps.toml — host, user, key name, port (no secrets)
2. deploy/cursor-vps.ps1 with subcommands: status | check | connect | fix-ssh
3. .vscode/tasks.json — "VPS: Status", "VPS: Connect", "VPS: Fix SSH"
4. .cursor/rules/vps.mdc — agents must use cursor-vps.ps1 check, not raw ssh
5. README section

## Requirements
- Quoted identity file path or cd-to-key-dir before ssh -i
- check exits: 0=ok, 2=ssh fail, 3=port closed, 4=health fail
- Remote probes via ssh: ss, curl, systemctl
- PowerShell only

Verify `.\deploy\cursor-vps.ps1 status` works and summarize usage in 3 lines.
```

---

## 短い追記命令（既存プロジェクト向け）

```
deploy/cursor-vps.ps1 を拡張してください。
- PostgreSQL 稼働も check に含める
- 失敗時に fix-start.sh を実行する -Repair スイッチを追加
```

```
.cursor/rules/vps.mdc を更新してください。
VPS 作業前は必ず .\deploy\cursor-vps.ps1 check を実行すること。
```

---

## この命令で Cursor が作る「型」

```
┌─────────────────────────────────────┐
│  Cursor / エージェント               │
│  .\deploy\cursor-vps.ps1 check      │
└──────────────┬──────────────────────┘
               │ SSH（鍵・config 自動）
               ▼
┌─────────────────────────────────────┐
│  VPS                                 │
│  ss / curl / systemctl              │
└──────────────┬──────────────────────┘
               │ JSON + exit code
               ▼
┌─────────────────────────────────────┐
│  Cursor が状態を理解                 │
│  SSH OK? 3000? aruaru health?       │
└─────────────────────────────────────┘
```

---

## ユーザーが覚えるコマンドは 1 つ

```powershell
.\deploy\cursor-vps.ps1 status
```

---

## 配布するときの一言

> 「Cursor に `docs/CURSOR_VPS_PROMPT.md` の命令文を貼ると、  
> VPS の SSH と状態確認が `cursor-vps.ps1 status` 1 本にまとまります。」

---

*aruaru project — share freely (MIT)*
