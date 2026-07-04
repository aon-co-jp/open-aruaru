use std::path::Path;
use std::process::{Command, Output, Stdio};

use anyhow::{bail, Context};

pub fn detect_remote(dir: &Path) -> anyhow::Result<String> {
    if !dir.join(".git").exists() {
        return Ok(String::new());
    }
    let out = git(dir, &["remote", "get-url", "origin"])?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Ok(String::new())
    }
}

pub fn push_folder(
    dir: &Path,
    remote_url: &str,
    branch: &str,
    message: &str,
    mut log: impl FnMut(String),
) -> anyhow::Result<()> {
    ensure_git()?;

    if !dir.is_dir() {
        bail!("フォルダが存在しません: {}", dir.display());
    }

    if !dir.join(".git").exists() {
        log("git init".into());
        run_git(dir, &["init"], &mut log)?;
        run_git(dir, &["branch", "-M", branch], &mut log)?;
    }

    set_remote(dir, remote_url, &mut log)?;

    log("git add -A".into());
    run_git(dir, &["add", "-A"], &mut log)?;

    let status = git(dir, &["status", "--porcelain"])?;
    if status.stdout.is_empty() {
        log("変更なし — push のみ試行".into());
    } else {
        let msg = if message.is_empty() {
            format!("Update {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
        } else {
            message.to_string()
        };
        log(format!("git commit -m \"{msg}\""));
        run_git(dir, &["commit", "-m", &msg], &mut log)?;
    }

    log(format!("git push -u origin {branch}"));
    let push = git(dir, &["push", "-u", "origin", branch])?;
    if !push.status.success() {
        let stderr = String::from_utf8_lossy(&push.stderr);
        let stdout = String::from_utf8_lossy(&push.stdout);
        bail!(
            "git push 失敗。\n\
             GitHub にリポジトリを作成済みか、PAT/SSH 認証を設定してください。\n\
             {stdout}{stderr}"
        );
    }

    Ok(())
}

fn ensure_git() -> anyhow::Result<()> {
    let out = Command::new("git")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("git コマンドが見つかりません。https://git-scm.com/download/win をインストールしてください")?;
    if !out.status.success() {
        bail!("git が利用できません");
    }
    Ok(())
}

fn set_remote(dir: &Path, remote_url: &str, log: &mut impl FnMut(String)) -> anyhow::Result<()> {
    let url = remote_url.trim();
    if url.is_empty() {
        bail!("GitHub URL が空です");
    }

    let has_origin = git(dir, &["remote", "get-url", "origin"])
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_origin {
        log(format!("git remote set-url origin {url}"));
        run_git(dir, &["remote", "set-url", "origin", url], log)?;
    } else {
        log(format!("git remote add origin {url}"));
        run_git(dir, &["remote", "add", "origin", url], log)?;
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> anyhow::Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("git {} (in {})", args.join(" "), dir.display()))
}

fn run_git(dir: &Path, args: &[&str], _log: &mut impl FnMut(String)) -> anyhow::Result<()> {
    let out = git(dir, args)?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        bail!("git {} 失敗:\n{stdout}{stderr}", args.join(" "));
    }
    Ok(())
}
