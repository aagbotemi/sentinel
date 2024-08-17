use csv::Writer;
use serde_json::Value;
use std::fs::File;

use crate::primitive::AppError;

pub fn trim_str(data: &Value) -> String {
    data.to_string().trim_matches('"').to_string()
}

pub fn hex_to_string(data: &Value) -> Result<u64, AppError> {
    if let Some(hex_str) = data.as_str() {
        let decimal = u64::from_str_radix(&hex_str[2..], 16).unwrap_or(0);
        Ok(decimal)
    } else {
        Err(AppError::Other("Block number is not a valid string".into()))
    }
}

pub fn num_to_string(data: &Option<u64>) -> String {
    data.map(|num| num.to_string())
        .unwrap_or_else(|| "No number found".to_string())
}

pub fn csv_writer() -> Result<Writer<File>, AppError> {
    let mut wtr: Writer<std::fs::File> = Writer::from_path("transaction_times.csv")?;
    // Write CSV headers
    wtr.write_record(&[
        "transaction_hash",
        "mempool_time (ms)",
        "gas_price",
        "block_number",
        "contract_type",
    ])?;
    wtr.flush()?;

    Ok(wtr)
}
