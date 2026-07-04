//! SFTP ファイルマネージャ（WinSCP 相当）
//! ブラウザからファイルの一覧・アップロード・ダウンロード・削除・パーミッション変更

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name:        String,
    pub path:        String,
    pub size:        u64,
    pub is_dir:      bool,
    pub permissions: String,
    pub modified:    String,
}

/// ディレクトリ一覧を取得
pub fn list_dir(sess: &ssh2::Session, path: &str) -> Result<Vec<FileEntry>> {
    let sftp = sess.sftp()?;
    let entries = sftp.readdir(std::path::Path::new(path))?;

    let mut files: Vec<FileEntry> = entries
        .into_iter()
        .map(|(p, stat)| {
            let name = p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_dir   = stat.is_dir();
            let size     = stat.size.unwrap_or(0);
            let perm     = format_permissions(stat.perm.unwrap_or(0));
            let modified = stat.mtime
                .map(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(t as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default())
                .unwrap_or_default();

            FileEntry {
                name,
                path:        p.to_string_lossy().to_string(),
                size,
                is_dir,
                permissions: perm,
                modified,
            }
        })
        .collect();

    // ディレクトリを先頭に、名前順でソート
    files.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
    });

    Ok(files)
}

/// ファイルをダウンロード（バイト列で返す）
pub fn download_file(sess: &ssh2::Session, remote_path: &str) -> Result<Vec<u8>> {
    let sftp = sess.sftp()?;
    let mut file = sftp.open(std::path::Path::new(remote_path))?;
    let mut buf = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// ファイルをアップロード
pub fn upload_file(sess: &ssh2::Session, remote_path: &str, data: &[u8]) -> Result<()> {
    let sftp = sess.sftp()?;
    let mut file = sftp.create(std::path::Path::new(remote_path))?;
    use std::io::Write;
    file.write_all(data)?;
    Ok(())
}

/// ファイル/ディレクトリを削除
pub fn delete_path(sess: &ssh2::Session, remote_path: &str, is_dir: bool) -> Result<()> {
    let sftp = sess.sftp()?;
    if is_dir {
        sftp.rmdir(std::path::Path::new(remote_path))?;
    } else {
        sftp.unlink(std::path::Path::new(remote_path))?;
    }
    Ok(())
}

/// パーミッションを rwxrwxrwx 形式に変換
fn format_permissions(perm: u32) -> String {
    let bits = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    bits.iter()
        .map(|(mask, ch)| if perm & mask != 0 { *ch } else { '-' })
        .collect()
}
