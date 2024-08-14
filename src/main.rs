use async_tungstenite::{tokio::connect_async, tungstenite::protocol::Message};
use csv::Writer;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    time::{interval, Duration},
};

#[derive(Deserialize)]
struct Params {
    result: String,
    subscription: String,
}

#[derive(Deserialize)]
struct TxHashResponse {
    jsonrpc: String,
    method: String,
    params: Params,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = env::var("WEB_SOCKET_URL")?;

    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connected");

    let (mut write, read) = ws_stream.split();

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

    // CSV writer
    let mut wtr = Writer::from_path("transaction_times.csv")?;

    // Write CSV headers
    wtr.write_record(&["Transaction Hash", "Mempool Time (ms)", "Block Number"])?;
    wtr.flush()?;

    // Ensure the responses directory exists
    fs::create_dir_all("responses").await?;

    // Create a ticker that fires every 30 seconds
    let mut interval = interval(Duration::from_secs(3));

    loop {
        tokio::select! {
            Some(message) = fused_read.next() => {
                match message {
                    Ok(Message::Text(res)) => {
                        if let Ok(response) = serde_json::from_str::<TxHashResponse>(&res) {
                            let tx_hash = response.params.result;
                            let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
                            tx_times.insert(tx_hash.clone(), start_time);
                            pending_txs.insert(tx_hash.clone());
                            println!("New pending transaction: {}", tx_hash);
                        }
                    }
                    Ok(Message::Binary(bin)) => println!("Received binary data: {:?}", bin),
                    Ok(Message::Close(_)) => {
                        println!("WebSocket closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            _ = interval.tick() => {
                println!("Checking pending transactions...");
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

                        dbg!(&tx_response);

                        if let Some(block_number) = tx_response["result"]["blockNumber"].as_str() {
                            if block_number != "null" {
                                let decimal = u64::from_str_radix(&block_number[2..], 16).unwrap_or(0);

                                if let Some(start_time) = tx_times.remove(&tx_hash) {
                                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

                                    let mempool_time = end_time - start_time;

                                    wtr.write_record(&[&tx_hash, &mempool_time.to_string(), &decimal.to_string()])?;
                                    wtr.flush()?;
                                    // println!("Transaction {} mined. Block: {}, Mempool time: {}ms", tx_hash, decimal, mempool_time);


                                    // Remove from pending set
                                    pending_txs.remove(&tx_hash);
                                }

                            }
                        };


                        // Save response to file
                        let file_path = format!("responses/{}.json", tx_hash);
                        let mut file = File::create(&file_path).await?;
                        file.write_all(response_text.as_bytes()).await?;


                    }
                }
                println!("Pending transactions check completed. Current pending count: {}", pending_txs.len());
            }
        }
    }

    wtr.flush()?;
    drop(wtr);

    Ok(())
}
