//! Web SSH ターミナル — WebSocket 経由でブラウザからSSH操作
//! TeraTerm/PuTTY 相当の機能をブラウザで実現

use anyhow::Result;
use russh::{client, ChannelMsg};
use russh_keys::key::PublicKey;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct SshClient;

impl client::Handler for SshClient {
    type Error = anyhow::Error;
    async fn check_server_key(
        &mut self, _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true) // 本番では known_hosts チェックを実装
    }
}

/// SSH 接続を確立して WebSocket ブリッジを開始
pub async fn connect_and_bridge(
    ip:          &str,
    port:        u16,
    username:    &str,
    private_key: Option<&str>,
    password:    Option<&str>,
    // ブラウザ → SSH への送信チャネル
    mut ws_rx:   mpsc::Receiver<String>,
    // SSH → ブラウザへの送信チャネル
    ws_tx:       mpsc::Sender<String>,
) -> Result<()> {
    let config = Arc::new(client::Config::default());
    let mut session = client::connect(config, (ip, port), SshClient).await?;

    // 認証
    if let Some(pem) = private_key {
        let key = russh_keys::decode_secret_key(pem, None)?;
        session.authenticate_publickey(username, Arc::new(key)).await?;
    } else if let Some(pw) = password {
        session.authenticate_password(username, pw).await?;
    }

    // シェルチャネル開始
    let mut channel = session.channel_open_session().await?;
    channel.request_pty(false, "xterm-256color", 200, 50, 0, 0, &[]).await?;
    channel.request_shell(false).await?;

    // 双方向ブリッジ（WebSocket ↔ SSH）
    loop {
        tokio::select! {
            // ブラウザからの入力 → SSH へ
            Some(input) = ws_rx.recv() => {
                channel.data(input.as_bytes()).await?;
            }
            // SSH からの出力 → ブラウザへ
            Some(msg) = channel.wait() => {
                match msg {
                    ChannelMsg::Data { data } => {
                        let text = String::from_utf8_lossy(&data).to_string();
                        let _ = ws_tx.send(text).await;
                    }
                    ChannelMsg::Eof | ChannelMsg::Close => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
    Ok(())
}

/// PTY リサイズ（ブラウザウィンドウリサイズ時）
pub async fn resize_pty(channel: &mut russh::Channel<russh::client::Msg>, cols: u32, rows: u32) -> Result<()> {
    channel.window_change(cols, rows, 0, 0).await?;
    Ok(())
}
