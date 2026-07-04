//! Web ターミナル（WebSocket + xterm.js）
//! ブラウザ内で TeraTerm 相当の SSH ターミナルを提供

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use poem::web::websocket::{Message, WebSocket};
use poem::{handler, web::Data, IntoResponse};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db::Pool;

/// WebSocket ハンドラ：ブラウザ ↔ SSH セッションを中継
#[handler]
pub async fn ws_terminal(
    ws:       WebSocket,
    Data(pool): Data<&Pool>,
    poem::web::Query(q): poem::web::Query<TerminalQuery>,
) -> impl IntoResponse {
    let pool = pool.clone();

    ws.on_upgrade(move |mut socket| async move {
        // VPS 情報を DB から取得
        let vps = match get_vps_info(&pool, &q.vps_id).await {
            Ok(v)  => v,
            Err(e) => {
                let _ = socket.send(Message::Text(
                    format!("\r\n\x1b[31m接続エラー: {}\x1b[0m\r\n", e)
                )).await;
                return;
            }
        };

        // SSH 接続
        let auth = build_auth(&pool, &vps).await;
        let sess = match crate::ssh::connect(&vps.ip, vps.port, &vps.user, &auth).await {
            Ok(s)  => Arc::new(Mutex::new(s)),
            Err(e) => {
                let _ = socket.send(Message::Text(
                    format!("\r\n\x1b[31mSSH 接続失敗: {}\x1b[0m\r\n", e)
                )).await;
                return;
            }
        };

        // PTY チャンネルを開く
        let channel = {
            let s = sess.lock().await;
            let mut ch = match s.channel_session() {
                Ok(c)  => c,
                Err(e) => {
                    let _ = socket.send(Message::Text(
                        format!("\r\n\x1b[31mチャンネル開放失敗: {}\x1b[0m\r\n", e)
                    )).await;
                    return;
                }
            };
            ch.request_pty("xterm-256color", None, Some((80, 24, 0, 0))).ok();
            ch.shell().ok();
            ch
        };

        let ch = Arc::new(Mutex::new(channel));

        let ch_read  = ch.clone();
        let ch_write = ch.clone();

        // SSH → WebSocket（VPS からの出力をブラウザに転送）
        let (mut ws_tx, mut ws_rx) = socket.split();

        let read_task = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                let ch = ch_read.blocking_lock();
                match ch.as_ref().read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let data = base64::encode(&buf[..n]);
                        // base64 でエンコードして送信（バイナリ安全）
                        drop(ch);
                        // 非同期ランタイムへ送信はチャンネル経由
                        let _ = std::io::stdout().write_all(data.as_bytes());
                    }
                }
            }
        });

        // WebSocket → SSH（ブラウザの入力を VPS に転送）
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    // JSON: {"type":"input","data":"..."} or {"type":"resize","cols":80,"rows":24}
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        match v["type"].as_str() {
                            Some("input") => {
                                if let Some(data) = v["data"].as_str() {
                                    let mut ch = ch_write.lock().await;
                                    let _ = ch.write_all(data.as_bytes());
                                }
                            }
                            Some("resize") => {
                                let cols = v["cols"].as_u64().unwrap_or(80) as u32;
                                let rows = v["rows"].as_u64().unwrap_or(24) as u32;
                                let mut ch = ch_write.lock().await;
                                let _ = ch.request_pty_size(cols, rows, Some(0), Some(0));
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }

        read_task.abort();
    })
}

#[derive(serde::Deserialize)]
pub struct TerminalQuery {
    pub vps_id: String,
}

struct VpsInfo {
    pub ip:      String,
    pub port:    u16,
    pub user:    String,
    pub auth_method: String,
    pub password:    Option<String>,
    pub keypair_id:  Option<String>,
}

async fn get_vps_info(pool: &Pool, vps_id: &str) -> Result<VpsInfo> {
    let r = sqlx::query!(
        "SELECT ip_address,ssh_port,ssh_user,auth_method,ssh_password,keypair_id
         FROM vps_servers WHERE id=?", vps_id
    )
    .fetch_one(pool)
    .await?;

    Ok(VpsInfo {
        ip:          r.ip_address,
        port:        r.ssh_port as u16,
        user:        r.ssh_user,
        auth_method: r.auth_method,
        password:    r.ssh_password,
        keypair_id:  r.keypair_id,
    })
}

async fn build_auth(pool: &Pool, vps: &VpsInfo) -> crate::ssh::SshAuth {
    if vps.auth_method == "password" {
        return crate::ssh::SshAuth::Password(
            vps.password.clone().unwrap_or_default()
        );
    }
    // 鍵認証
    if let Some(kid) = &vps.keypair_id {
        if let Ok(r) = sqlx::query!(
            "SELECT private_key FROM ssh_keypairs WHERE id=?", kid
        ).fetch_one(pool).await {
            return crate::ssh::SshAuth::PrivateKey {
                private_key_pem: r.private_key,
                passphrase:      None,
            };
        }
    }
    crate::ssh::SshAuth::Password(String::new())
}
