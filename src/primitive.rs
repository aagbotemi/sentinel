use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}
};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool, Type};
use std::env;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Params {
    pub result: String,
    pub subscription: String,
}

#[derive(Debug, Deserialize)]
pub struct TxHashResponse {
    pub jsonrpc: String,
    pub method: String,
    pub params: Params,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: i64,
    pub from_sender: String,
    pub to_reciever: String,
    pub tx_value: i64,
    pub gas: i64,
    pub gas_price: i64,
    pub input: String,
    pub nonce: i64,
    pub mempool_time: i64, // time spent in the mempool
    pub contract_type: ContractType,
}

#[derive(Deserialize)]
pub struct TransactionFilter {
    pub gas_price_min: Option<i64>,
    pub gas_price_max: Option<i64>,
    pub contract_type: Option<String>,
    pub block_number_min: Option<i64>,
    pub block_number_max: Option<i64>,
    pub mempool_time_min: Option<i64>,
    pub mempool_time_max: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub web_socket_url: String,
    pub db_url: String,
    pub server_url: String,
}

pub struct AppState {
    pub pool: PgPool,
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
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Not found error: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::WebSocketError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::JsonError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::CsvError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::EnvVarError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Other(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
        };

        (status, error_message).into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Type)]
#[sqlx(type_name = "contract_type", rename_all = "lowercase")]
pub enum ContractType {
    ExternallyOwnedAccount,
    ContractAccount,
    SpecialCaseContract, // possible a proxy or upgradable contract
}
