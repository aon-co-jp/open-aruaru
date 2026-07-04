//! SSH / SFTP クライアント + Web ターミナル
//! TeraTerm / WinSCP 相当の機能をブラウザで提供

pub mod keygen;
pub mod terminal;
pub mod sftp;

use anyhow::{Context, Result};
use ssh2::Session;
use std::net::TcpStream;
use std::time::Duration;

/// SSH セッションを確立する
pub async fn connect(
    ip:       &str,
    port:     u16,
    user:     &str,
    auth:     &SshAuth,
) -> Result<Session> {
    let addr = format!("{}:{}", ip, port);

    // TCP 接続（タイムアウト 10 秒）
    let tcp = TcpStream::connect_timeout(
        &addr.parse().context("IP アドレスの形式が不正です")?,
        Duration::from_secs(10),
    )
    .context(format!("VPS {}:{} に接続できません", ip, port))?;

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // 認証
    match auth {
        SshAuth::Password(pw) => {
            sess.userauth_password(user, pw)
                .context("パスワード認証に失敗しました")?;
        }
        SshAuth::PrivateKey { private_key_pem, passphrase } => {
            // 一時ファイルに書き出して認証
            let tmp = std::env::temp_dir().join(format!("aruaru_key_{}.pem", uuid::Uuid::new_v4()));
            std::fs::write(&tmp, private_key_pem)?;
            let pp = passphrase.as_deref();
            let result = sess.userauth_pubkey_file(user, None, &tmp, pp);
            let _ = std::fs::remove_file(&tmp); // 即削除
            result.context("SSH 鍵認証に失敗しました")?;
        }
    }

    if !sess.authenticated() {
        anyhow::bail!("SSH 認証に失敗しました");
    }

    Ok(sess)
}

/// コマンドを実行して出力を返す
pub fn exec(sess: &Session, cmd: &str) -> Result<(String, String, i32)> {
    let mut ch = sess.channel_session()?;
    ch.exec(cmd)?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    use std::io::Read;
    ch.read_to_string(&mut stdout)?;
    ch.stderr().read_to_string(&mut stderr)?;
    ch.wait_close()?;
    let exit = ch.exit_status()?;
    Ok((stdout, stderr, exit))
}

#[derive(Debug, Clone)]
pub enum SshAuth {
    Password(String),
    PrivateKey {
        private_key_pem: String,
        passphrase:      Option<String>,
    },
}
