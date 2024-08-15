use log::error;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Deserialize)]
pub struct Params {
    pub result: String,
    pub subscription: String,
}

#[derive(Deserialize)]
pub struct TxHashResponse {
    pub jsonrpc: String,
    pub method: String,
    pub params: Params,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub tx_hash: String,
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub from: String,
    pub to: Option<String>,
    pub value: u64,
    pub gas: u64,
    pub gas_price: u64,
    pub input: String,
    pub nonce: u64,
    pub mempool_time: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub web_socket_url: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] async_tungstenite::tungstenite::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Environment variable not found: {0}")]
    EnvVarError(#[from] env::VarError),
    #[error("Other error: {0}")]
    Other(String),
}
