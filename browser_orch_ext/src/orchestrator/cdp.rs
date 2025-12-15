use anyhow::{anyhow, Result};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
    StreamExt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Debug, sync::atomic::AtomicI64};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{Request, Response};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CdpMessage {
    pub id: Option<i64>,
    pub method: String,
    pub params: Value,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CdpResponse {
    Message(CdpMessage),
    Response(Value),
}

/// A connection to a CDP server.
pub struct CdpConnection {
    #[allow(dead_code)]
    request: Request<()>,    
    ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    counter: AtomicI64,
}

impl CdpConnection {
    pub async fn new(mut url: String) -> Result<(Self, Response<Option<Vec<u8>>>)> {
        if !url.starts_with("ws://") && !url.starts_with("wss://") {
            let resp = reqwest::get(format!("http://{}/json/version", url).replace("ws://", ""))
                .await?;

            let json: serde_json::Value = resp.json().await?;
            url = json["webSocketDebuggerUrl"].as_str().unwrap().to_owned();
        }

        let request = url.into_client_request()?;

        let (ws, resp) = connect_async(request.clone()).await?;

        let (ws_sender, ws_receiver) = ws.split();

        Ok((
            Self {
                ws_sender,
                ws_receiver,
                request,
                counter: AtomicI64::new(0),
            },
            resp,
        ))
    }

    pub async fn send_message<T: Serialize>(
        &mut self,
        method: &str,
        params: T,
    ) -> Result<serde_json::Value> {
        let id = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let message = serde_json::to_string(&serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        }))?;

        self.ws_sender.send(Message::Text(message)).await?;

        loop {
            let Some(msg) = self.ws_receiver.next().await else {
                continue;
            };
            let msg = msg?;

            match msg {
                Message::Text(text) => {
                    let json: serde_json::Value = serde_json::from_str(&text)?;
                    if let Some(msg_id) = json.get("id").and_then(|id| id.as_i64()) {
                        if msg_id == id {
                            return Ok(json.get("result").unwrap().clone());
                        }
                    }
                }
                Message::Close(_) => {
                    return Err(anyhow!("Connection closed"));
                }
                _ => {}
            }
        }
    }

    pub async fn get_page_state(&mut self) -> Result<Value, anyhow::Error> {
        let state_js = include_str!("./get_state.js");

        let result = self.send_message(
            "Runtime.evaluate",
            serde_json::json!({
                "expression": state_js,
                "awaitPromise": true,
                "returnByValue": true,
            }),
        )
        .await?;
        Ok(result)
    }
}
