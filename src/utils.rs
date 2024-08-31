use crate::model::AppError;
use alloy::primitives::ChainId;
use csv::{Writer, WriterBuilder};
use serde_json::Value;
use std::{
    env::var,
    fs::{File, OpenOptions},
    path::Path,
};

pub fn trim_str(data: &Value) -> String {
    data.to_string().trim_matches('"').to_string()
}

pub fn hex_to_int64(data: &Value) -> Result<i64, AppError> {
    if let Some(hex_str) = data.as_str() {
        let decimal = i64::from_str_radix(&hex_str[2..], 16).unwrap_or(0);
        Ok(decimal)
    } else {
        Err(AppError::Other("Block number is not a valid string".into()))
    }
}

pub fn csv_writer(file_path: &str) -> Result<Writer<File>, std::io::Error> {
    let path = Path::new(file_path);
    let file_exists = path.exists();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_path)?;

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

    if !file_exists {
        wtr.write_record(&[
            "transaction_hash",
            "mempool_time (ms)",
            "gas_price",
            "block_number",
            "contract_type",
        ])?;
    }

    Ok(wtr)
}

pub fn get_rpc_url_with_chain_id(chain_id: ChainId) -> String {
    match chain_id {
        1 => var("MAINNET_WEB_SOCKET_URL").unwrap(),
        11155111 => var("SEPOLIA_WEB_SOCKET_URL").unwrap(),
        534352 => var("SCROLL_WEB_SOCKET_URL").unwrap(),
        534351 => var("SCROLL_SEPOLIA_WEB_SOCKET_URL").unwrap(),
        8453 => var("BASE_WEB_SOCKET_URL").unwrap(),
        84531 => var("BASE_SEPOLIA_WEB_SOCKET_URL").unwrap(),
        10 => var("OP_WEB_SOCKET_URL").unwrap(),
        420 => var("OP_SEPOLIA_WEB_SOCKET_URL").unwrap(),
        56 => var("BINANCE_WEB_SOCKET_URL").unwrap(),
        137 => var("POLYGON_POS_WEB_SOCKET_URL").unwrap(),
        1101 => var("POLYGON_ZKEVM_WEB_SOCKET_URL").unwrap(),
        42161 => var("ARBITRUMONE_WEB_SOCKET_URL").unwrap(),
        421614 => var("ARBITRUM_SEPOLIA_WEB_SOCKET_URL").unwrap(),
        324 => var("ZKSYNC_WEB_SOCKET_URL").unwrap(),
        // sn_main => var("STARKNET_WEB_SOCKET_URL").unwrap(),
        // sn_sepolia => var("STARKNET_SEPOLIA_WEB_SOCKET_URL").unwrap(),
        // 900 => var("SOLANA_WEB_SOCKET_URL").unwrap(),
        _ => "No corresponding rpc url for given chain Id".to_owned(),
    }
}
