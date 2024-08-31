use crate::model::{AppState, Transaction, TransactionFilter};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use std::sync::Arc;

#[derive(SimpleObject)]
struct GraphQLTransaction {
    id: String,
    tx_hash: String,
    block_hash: String,
    block_number: i64,
    from_sender: String,
    to_reciever: String,
    tx_value: i64,
    gas: i64,
    gas_price: i64,
    input: String,
    nonce: i64,
    mempool_time: i64,
    contract_type: String,
}

pub struct Query;

#[Object]
impl Query {
    async fn get_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<GraphQLTransaction>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let transactions = sqlx::query_as::<_, Transaction>("SELECT * FROM transaction")
            .fetch_all(&state.pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(transactions
            .into_iter()
            .map(|t| GraphQLTransaction {
                id: t.id.to_string(),
                tx_hash: t.tx_hash,
                block_hash: t.block_hash,
                block_number: t.block_number,
                from_sender: t.from_sender,
                to_reciever: t.to_reciever,
                tx_value: t.tx_value,
                gas: t.gas,
                gas_price: t.gas_price,
                input: t.input,
                nonce: t.nonce,
                mempool_time: t.mempool_time,
                contract_type: t.contract_type.as_str().to_string(),
            })
            .collect())
    }

    async fn get_transaction(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> async_graphql::Result<GraphQLTransaction> {
        let state = ctx.data::<Arc<AppState>>()?;
        let transaction =
            sqlx::query_as::<_, Transaction>("SELECT * FROM transaction WHERE id = $1")
                .bind(uuid::Uuid::parse_str(&id)?)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
                .ok_or_else(|| async_graphql::Error::new("Transaction not found"))?;

        Ok(GraphQLTransaction {
            id: transaction.id.to_string(),
            tx_hash: transaction.tx_hash,
            block_hash: transaction.block_hash,
            block_number: transaction.block_number,
            from_sender: transaction.from_sender,
            to_reciever: transaction.to_reciever,
            tx_value: transaction.tx_value,
            gas: transaction.gas,
            gas_price: transaction.gas_price,
            input: transaction.input,
            nonce: transaction.nonce,
            mempool_time: transaction.mempool_time,
            contract_type: transaction.contract_type.as_str().to_string(),
        })
    }

    async fn filter_transactions(
        &self,
        ctx: &Context<'_>,
        filter: TransactionFilter,
    ) -> async_graphql::Result<Vec<GraphQLTransaction>> {
        let state = ctx.data::<Arc<AppState>>()?;

        let mut query = String::from("SELECT * FROM transactions WHERE 1=1");
        let mut bindings = vec![];

        if let Some(min) = filter.gas_price_min {
            query += " AND gas_price >= $1";
            bindings.push(min.to_string());
        }

        if let Some(max) = filter.gas_price_max {
            query += &format!(" AND gas_price <= ${}", bindings.len() + 1);
            bindings.push(max.to_string());
        }

        if let Some(contract_type) = filter.contract_type {
            query += &format!(" AND contract_type = ${}", bindings.len() + 1);
            bindings.push(contract_type);
        }

        if let Some(min) = filter.block_number_min {
            query += &format!(" AND block_number >= ${}", bindings.len() + 1);
            bindings.push(min.to_string());
        }

        if let Some(max) = filter.block_number_max {
            query += &format!(" AND block_number <= ${}", bindings.len() + 1);
            bindings.push(max.to_string());
        }

        if let Some(min) = filter.mempool_time_min {
            query += &format!(" AND mempool_time >= ${}", bindings.len() + 1);
            bindings.push(min.to_string());
        }

        if let Some(max) = filter.mempool_time_max {
            query += &format!(" AND mempool_time <= ${}", bindings.len() + 1);
            bindings.push(max.to_string());
        }

        let mut db_query = sqlx::query_as::<_, Transaction>(&query);
        for binding in bindings {
            db_query = db_query.bind(binding);
        }

        let transactions = db_query
            .fetch_all(&state.pool)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(transactions
            .into_iter()
            .map(|t| GraphQLTransaction {
                id: t.id.to_string(),
                tx_hash: t.tx_hash,
                block_hash: t.block_hash,
                block_number: t.block_number,
                from_sender: t.from_sender,
                to_reciever: t.to_reciever,
                tx_value: t.tx_value,
                gas: t.gas,
                gas_price: t.gas_price,
                input: t.input,
                nonce: t.nonce,
                mempool_time: t.mempool_time,
                contract_type: t.contract_type.as_str().to_string(),
            })
            .collect())
    }
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema(state: Arc<AppState>) -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(state)
        .finish()
}
