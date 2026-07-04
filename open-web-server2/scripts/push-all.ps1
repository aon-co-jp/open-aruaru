# push-all.ps1 — f:\open-aruaru 配下の各リポジトリをまとめて git init → commit → push する。
#
# 前提:
#   - 各サブディレクトリ名と同名のGitHubリポジトリを事前に作成しておくこと
#   - git がインストール済みで、SSH鍵 or GitHub CLI (gh auth login) で認証済みであること
#
# 使い方 (PowerShellで f:\open-aruaru 直下から):
#   .\open-web-server\scripts\push-all.ps1 -GitHubUser your-account
#
# 個別リポジトリだけ対象にしたい場合:
#   .\push-all.ps1 -GitHubUser your-account -Repos open-web-server,open-runo

param(
    [Parameter(Mandatory = $true)]
    [string]$GitHubUser,

    [string[]]$Repos = @("open-web-server", "open-runo", "aruaru-db", "open-client", "open-aruaru")
)

foreach ($repo in $Repos) {
    if (-not (Test-Path $repo)) {
        Write-Warning "スキップ: $repo ディレクトリが見つかりません"
        continue
    }

    Write-Host "== $repo =="
    Push-Location $repo
    try {
        if (-not (Test-Path ".git")) {
            git init -b main
            git remote add origin "git@github.com:$GitHubUser/$repo.git" 2>$null
        }

        if (-not (Test-Path ".gitignore")) {
            @"
/target
/data/
!/data/.gitkeep
Cargo.lock
node_modules/
"@ | Out-File -Encoding utf8 ".gitignore"
        }

        git add -A
        $diff = git diff --cached --name-only
        if ([string]::IsNullOrWhiteSpace($diff)) {
            Write-Host "  変更なし。push をスキップします。"
        } else {
            git commit -m "sync: update from f:\open-aruaru ($(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))"
        }

        git push -u origin main
    }
    finally {
        Pop-Location
    }
    Write-Host ""
}

Write-Host "完了。対象: $($Repos -join ', ')"
