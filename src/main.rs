use dotenv::dotenv;
use log::error;
use std::error::Error;
use tokio::{
    fs::{self},
    time::Duration,
};

use sentinel::{connection::load_config, mempool::mempool::scan_mempool, primitive::AppError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Config
    let config = load_config()?;
    // Ensure the responses directory exists
    fs::create_dir_all("responses").await?;

    loop {
        let result: Result<(), AppError> = async {
            scan_mempool(&config).await?;
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("Error occurred: {:?}", e);
            error!("Reconnecting in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
