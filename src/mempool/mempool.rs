use async_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{fs::File, io::AsyncWriteExt, time::interval};

use crate::{
    connection::connect_websocket,
    mempool::check_contract_type::is_contract_account,
    primitive::{AppError, Config, ContractType, Transaction, TxHashResponse},
    utils::{csv_writer, hex_to_string, num_to_string, trim_str},
};

pub async fn scan_mempool(config: &Config) -> Result<(), AppError> {
    let (mut write, read) = connect_websocket(&config.web_socket_url).await?.split();

    // CSV writer
    let mut wtr = csv_writer()?;

    // Subscribe to pending transactions
    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newPendingTransactions"]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;
    let mut fused_read = read.fuse();

    // HashMap to store transaction times
    let mut tx_times: HashMap<String, u64> = HashMap::new();
    let mut pending_txs: HashSet<String> = HashSet::new();
    // Create a ticker that fires every 3 seconds
    let mut interval = interval(Duration::from_secs(3));
    loop {
        tokio::select! {
            Some(message) = fused_read.next() => {
                match message {
                    Ok(Message::Text(res)) => {
                        if let Ok(response) = serde_json::from_str::<TxHashResponse>(&res) {
                            let tx_hash = response.params.result;
                            let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                            tx_times.insert(tx_hash.clone(), start_time);
                            pending_txs.insert(tx_hash.clone());
                            info!("New pending transaction: {}", tx_hash);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("WebSocket closed");
                        return Err(AppError::Other("WebSocket closed".into()));
                    }
                    Err(e) => {
                        error!("Error: {}", e);
                        return Err(AppError::WebSocketError(e));
                    }
                    _ => {}
                }
            }
            _ = interval.tick() => {
                info!("Checking pending transactions...");

                for tx_hash in pending_txs.clone() {
                    let tx_data = json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "eth_getTransactionByHash",
                        "params": [&tx_hash]
                    });
                    write.send(Message::Text(tx_data.to_string())).await?;

                    if let Some(Ok(Message::Text(response_text))) = fused_read.next().await {
                        let tx_response: Value = serde_json::from_str(&response_text)?;

                        if let Some(result) = tx_response.get("result") {
                            if result["blockHash"].is_string() {
                                let check_contract_code = json!({
                                    "jsonrpc": "2.0",
                                    "id": 1,
                                    "method": "eth_getCode",
                                    "params": [&result["to"]]
                                });

                                write.send(Message::Text(check_contract_code.to_string())).await?;
                                let mut _contract_type: ContractType = ContractType::ExternallyOwnedAccount;

                                // check the contract type
                                if let Some(Ok(Message::Text(check_contract_code_response_text))) = fused_read.next().await {
                                    let check_contract_code_tx_response: Value = serde_json::from_str(&check_contract_code_response_text)?;
                                    let code = &check_contract_code_tx_response["result"];


                                    _contract_type = is_contract_account(&code);
                                }

                                if let Some(start_time) = tx_times.remove(&tx_hash) {
                                    let block_number = hex_to_string(&result["blockNumber"]);
                                    let gas_price = hex_to_string(&result["gasPrice"]);
                                    let gas = hex_to_string(&result["gas"]);
                                    let value = hex_to_string(&result["value"]);
                                    let nonce = hex_to_string(&result["nonce"]);

                                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
                                    let mempool_time = end_time - start_time;

                                    let transaction = Transaction {
                                        tx_hash: tx_hash.clone(),
                                        block_hash:Some(trim_str(&result["blockHash"])),
                                        block_number: Some(block_number?),
                                        from: trim_str(&result["from"]),
                                        to: Some(trim_str(&result["to"])),
                                        value: value?,
                                        gas: gas?,
                                        gas_price: gas_price?,
                                        input: trim_str(&result["input"]),
                                        nonce: nonce?,
                                        mempool_time: Some(mempool_time),
                                        contract_type: _contract_type,
                                    };

                                    // Convert block_number to a String
                                    let blck_number_str = num_to_string(&transaction.block_number);
                                    wtr.write_record(&[&tx_hash, &mempool_time.to_string(), &transaction.gas_price.to_string(), &blck_number_str, &_contract_type.as_str().to_string()])?;
                                    // Remove from pending txn
                                    pending_txs.remove(&tx_hash);

                                    // Save response to file
                                    let file_path = format!("responses/{}.json", tx_hash);
                                    let mut file = File::create(&file_path).await?;
                                    file.write_all(serde_json::to_string(&transaction)?.as_bytes()).await?;
                                    wtr.flush()?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::check_contract_type::is_contract_account;

    #[test]
    fn test_is_contract_account() {
        assert_eq!(
            is_contract_account(&Value::Null),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            is_contract_account(&Value::String("0x".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            is_contract_account(&Value::String("0x0".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            is_contract_account(&Value::String("".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            is_contract_account(&Value::String("0x123".to_string())),
            ContractType::ContractAccount
        );
        assert_eq!(
            is_contract_account(&Value::String("{...}".to_string())),
            ContractType::SpecialCaseContract
        );
        assert_eq!(
            is_contract_account(&Value::Object(serde_json::Map::new())),
            ContractType::SpecialCaseContract
        );
    }
}
