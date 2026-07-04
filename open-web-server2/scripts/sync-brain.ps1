# sync-brain.ps1 — open-web-server の「学習済みの賢さ」を Googleドライブへ同期する (Windows版)
#
# 前提: rclone.exe が PATH に通っていること。`rclone config` で "gdrive" という
#       名前のリモートを一度だけ作成しておくこと (winget install Rclone.Rclone)。
#
# 使い方 (PowerShell):
#   .\sync-brain.ps1 push    # ローカル → Googleドライブ
#   .\sync-brain.ps1 pull    # Googleドライブ → ローカル (OS再インストール後の復元)
#
# タスクスケジューラ登録例 (5分毎に自動バックアップ):
#   $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-File C:\open-aruaru\open-web-server\scripts\sync-brain.ps1 push"
#   $trigger = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Minutes 5)
#   Register-ScheduledTask -TaskName "aruaru-brain-sync" -Action $action -Trigger $trigger

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("push", "pull")]
    [string]$Mode
)

$Remote = "gdrive:open-aruaru-brain"
$LocalDir = if ($env:ARUARU_DATA_DIR) { $env:ARUARU_DATA_DIR } else { ".\data" }
$BrainFile = "aruaru-brain.json"

if (-not (Get-Command rclone -ErrorAction SilentlyContinue)) {
    Write-Error "rclone が見つかりません。 'winget install Rclone.Rclone' を実行してください。"
    exit 1
}

switch ($Mode) {
    "push" {
        $src = Join-Path $LocalDir $BrainFile
        if (-not (Test-Path $src)) {
            Write-Warning "$src がまだ存在しません (100リクエスト到達前は未生成)。"
            exit 0
        }
        rclone copy $src "$Remote/" --checksum
        Write-Host "OK: $BrainFile を $Remote へ同期しました。"
    }
    "pull" {
        New-Item -ItemType Directory -Force -Path $LocalDir | Out-Null
        rclone copy "$Remote/$BrainFile" $LocalDir --checksum
        Write-Host "OK: $Remote から $BrainFile を復元しました。次回 open-web-server 起動時に自動で読み込まれます。"
    }
}
