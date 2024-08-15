use async_tungstenite::{
    tokio::{connect_async, ConnectStream},
    WebSocketStream,
};
use log::info;
use std::env;

use crate::data_structure::{AppError, Config};

pub fn load_config() -> Result<Config, AppError> {
    Ok(Config {
        web_socket_url: env::var("WEB_SOCKET_URL").unwrap(),
    })
}

pub async fn connect_websocket(url: &str) -> Result<WebSocketStream<ConnectStream>, AppError> {
    let (ws_stream, _) = connect_async(url).await?;
    info!("WebSocket connected");
    Ok(ws_stream)
}
