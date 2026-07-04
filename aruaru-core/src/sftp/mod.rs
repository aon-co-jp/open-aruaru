//! Web SFTP ファイルマネージャー — WinSCP 相当をブラウザで実現
//! アップロード・ダウンロード・ファイル一覧・削除・移動

use anyhow::Result;
use russh::client;
use russh_sftp::client::SftpSession;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use crate::ssh::SshClient;

/// SFTP セッションを確立
pub async fn connect(
    ip:          &str,
    port:        u16,
    username:    &str,
    private_key: Option<&str>,
    password:    Option<&str>,
) -> Result<SftpSession> {
    let config  = Arc::new(client::Config::default());
    let mut ssh = client::connect(config, (ip, port), SshClient).await?;

    if let Some(pem) = private_key {
        let key = russh_keys::decode_secret_key(pem, None)?;
        ssh.authenticate_publickey(username, Arc::new(key)).await?;
    } else if let Some(pw) = password {
        ssh.authenticate_password(username, pw).await?;
    }

    let channel = ssh.channel_open_session().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;
    Ok(sftp)
}

/// リモートディレクトリ一覧を取得
pub async fn list_dir(sftp: &SftpSession, path: &str) -> Result<Vec<FileEntry>> {
    let mut dir = sftp.read_dir(path).await?;
    let mut entries = Vec::new();
    while let Some(entry) = dir.next().await? {
        let metadata = entry.metadata();
        entries.push(FileEntry {
            name:        entry.file_name(),
            path:        format!("{}/{}", path.trim_end_matches('/'), entry.file_name()),
            is_dir:      metadata.is_dir(),
            size:        metadata.size.unwrap_or(0),
            modified:    metadata.mtime.map(|t| t.to_string()).unwrap_or_default(),
            permissions: metadata.permissions.map(|p| format!("{:o}", p)).unwrap_or_default(),
        });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(entries)
}

/// ファイルをダウンロード（リモート → バイト列）
pub async fn download(sftp: &SftpSession, remote_path: &str) -> Result<Vec<u8>> {
    let mut file = sftp.open(remote_path).await?;
    let mut buf  = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(buf)
}

/// ファイルをアップロード（バイト列 → リモート）
pub async fn upload(sftp: &SftpSession, remote_path: &str, data: &[u8]) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut file = sftp.create(remote_path).await?;
    file.write_all(data).await?;
    Ok(())
}

/// リモートファイル・ディレクトリ削除
pub async fn remove(sftp: &SftpSession, path: &str, is_dir: bool) -> Result<()> {
    if is_dir { sftp.remove_dir(path).await? } else { sftp.remove_file(path).await? }
    Ok(())
}

/// リネーム / 移動
pub async fn rename(sftp: &SftpSession, from: &str, to: &str) -> Result<()> {
    sftp.rename(from, to, None).await?;
    Ok(())
}

/// ディレクトリ作成
pub async fn mkdir(sftp: &SftpSession, path: &str) -> Result<()> {
    sftp.create_dir(path).await?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct FileEntry {
    pub name:        String,
    pub path:        String,
    pub is_dir:      bool,
    pub size:        u64,
    pub modified:    String,
    pub permissions: String,
}
