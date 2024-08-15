use async_tungstenite::tungstenite::protocol::Message;
use csv::Writer;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use sentinel::{
    connection::{connect_websocket, load_config},
    data_structure::{AppError, Transaction, TxHashResponse},
    utils::{hex_to_string, num_to_string, trim_str},
};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    time::{interval, Duration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let config = load_config()?;

    // CSV writer
    let mut wtr = Writer::from_path("transaction_times.csv")?;
    // Write CSV headers
    wtr.write_record(&[
        "Transaction Hash",
        "Mempool Time (ms)",
        "Gas Price",
        "Block Number",
    ])?;
    wtr.flush()?;

    // Ensure the responses directory exists
    fs::create_dir_all("responses").await?;

    loop {
        let result: Result<(), AppError> = async {
            let (mut write, read) = connect_websocket(&config.web_socket_url).await?.split();

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
                                let tx_response: Value = serde_json::from_str(&response_text).unwrap();

                                if let Some(result) = tx_response.get("result") {
                                    if result["blockHash"].is_string() {
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
                                                mempool_time: Some(mempool_time)
                                            };

                                            // Convert block_number to a String
                                            let blck_number_str = num_to_string(&transaction.block_number);
                                            wtr.write_record(&[&tx_hash, &mempool_time.to_string(), &transaction.gas_price.to_string(), &blck_number_str]).unwrap();
                                            // Remove from pending set
                                            pending_txs.remove(&tx_hash);

                                            // Save response to file
                                            let file_path = format!("responses/{}.json", tx_hash);
                                            let mut file = File::create(&file_path).await?;
                                            // println!("Transaction {} mined. Block: {}, Mempool time: {}ms", tx_hash, decimal, mempool_time);
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
            .await;

        if let Err(e) = result {
            error!("Error occurred: {:?}", e);
            error!("Reconnecting in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn mempool() {}
