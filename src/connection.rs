use crate::model::{AppError, Config};
use async_tungstenite::{
    tokio::{connect_async, ConnectStream},
    WebSocketStream,
};
use log::info;
use std::env::var;

pub fn load_config() -> Result<Config, AppError> {
    Ok(Config {
        web_socket_url: var("SEPOLIA_WEB_SOCKET_URL").unwrap(),
        db_url: var("DATABASE_URL").unwrap(),
        server_url: var("SERVER_ADDRESS").unwrap_or("127.0.0.1::3000".to_string()),
    })
}

pub async fn connect_websocket(url: &str) -> Result<WebSocketStream<ConnectStream>, AppError> {
    let (ws_stream, _) = connect_async(url).await?;
    info!("WebSocket connected");
    Ok(ws_stream)
}
