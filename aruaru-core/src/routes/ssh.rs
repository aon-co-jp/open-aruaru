//! Web SSH ターミナル WebSocket エンドポイント

use poem::{
    handler, get, post,
    web::{websocket::{WebSocket, WebSocketUpgrade, Message}, Data, Json, Path},
    Route, Response, Result, http::StatusCode, IntoResponse,
};
use serde::Deserialize;
use tokio::sync::mpsc;
use crate::{db::Pool, keygen, ssh, vps};

/// GET /api/ssh/:vps_id/terminal — WebSocket で SSH ターミナル
#[handler]
async fn terminal(
    Data(pool): Data<&Pool>,
    Path(vps_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse> {
    let pool = pool.clone();
    Ok(ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        // VPS 情報を取得
        let server = match vps::get(&pool, &vps_id).await {
            Ok(Some(s)) => s,
            _ => { let _ = sink.send(Message::Text("ERROR: VPS not found".into())).await; return; }
        };

        // 秘密鍵を取得
        let private_key = if let Some(kid) = &server.key_id {
            keygen::get_private_key(&pool, kid).await.ok().flatten()
        } else { None };

        let (ws_tx, mut ws_rx_inner) = mpsc::channel::<String>(256);
        let (ssh_tx, ssh_rx)         = mpsc::channel::<String>(256);

        // ブラウザ → SSH 転送タスク
        let pool2 = pool.clone();
        let vps_id2 = vps_id.clone();
        tokio::spawn(async move {
            ssh::connect_and_bridge(
                &server.ip_address,
                server.port as u16,
                &server.username,
                private_key.as_deref(),
                None,
                ssh_rx,
                ws_tx,
            ).await.ok();
        });

        // WebSocket メッセージ受信 → SSH へ
        tokio::spawn(async move {
            use futures_util::StreamExt;
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    let _ = ssh_tx.send(text).await;
                }
            }
        });

        // SSH 出力 → WebSocket へ送信
        while let Some(output) = ws_rx_inner.recv().await {
            if sink.send(Message::Text(output)).await.is_err() { break; }
        }
    }))
}

pub fn ssh_routes(pool: Pool) -> Route {
    Route::new()
        .at("/:vps_id/terminal", get(terminal).data(pool.clone()))
}
