use csv::{Writer, WriterBuilder};
use serde_json::Value;
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use crate::primitive::AppError;

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
