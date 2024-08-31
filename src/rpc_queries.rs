//! This module uses alloy to query the blockchain for information.

use alloy::{
    primitives::{Address, TxHash, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::eth::{Block, BlockId, Transaction},
    sol,
};
use std::error::Error;

// Codegen from artifact.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20Abi,
    "src/abi/ERC20Abi.json",
);

pub async fn get_block_query(rpc_url: String, block_id: BlockId) -> Result<Block, Box<dyn Error>> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?)?;
    let block = provider.get_block(block_id, false).await?.unwrap();

    Ok(block)
}

pub async fn get_transaction_query(
    rpc_url: String,
    tx_hash: TxHash,
) -> Result<Transaction, Box<dyn Error>> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?)?;
    let transaction = provider.get_transaction_by_hash(tx_hash).await?;

    Ok(transaction)
}

pub async fn get_native_balance_query(
    rpc_url: String,
    user_address: Address,
) -> Result<U256, Box<dyn Error>> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?)?;
    let balance = provider
        .get_balance(user_address, BlockId::latest())
        .await?;

    Ok(balance)
}

pub async fn get_erc20_balance_query(
    rpc_url: String,
    user_address: Address,
    contract_address: Address,
) -> Result<U256, Box<dyn Error>> {
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?)?;
    let contract = ERC20Abi::new(contract_address, provider);
    let balance = contract.balanceOf(user_address).call().await?._0;
    Ok(balance)
}
