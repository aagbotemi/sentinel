use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    http::Method, response::IntoResponse, routing::{get, post}, serve, Extension, Router
};
use dotenv::dotenv;
use log::error;
use sentinel::{
    connection::load_config,
    graphql::schema::{create_schema, AppSchema},
    mempool::mempool::scan_mempool,
    model::{AppError, AppState},
    service::{
        create_transaction, filter_transactions, get_block, get_erc20_balance, get_native_balance,
        get_transaction, get_transaction_by_id, get_transactions,
    },
};
use sqlx::postgres::PgPoolOptions;
use std::{error::Error, sync::Arc};
use tokio::{
    fs::{self},
    net::TcpListener,
    signal, task,
    time::Duration,
};
use tower_http::cors::{Any, CorsLayer};

#[axum::debug_handler]
async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    axum::response::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // CORS configuration allowing any origin, method, headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Config
    let config = load_config()?;

    // initialize the database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await
        .expect("Error connecting to Postgres");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = Arc::new(AppState { pool });

    let schema = create_schema(app_state.clone());

    let app = Router::new()
        .route(
            "/",
            get(|| async { "Sentinel! A blockchain indexing tool." }),
        )
        .route("/transactions", get(get_transactions))
        .route("/transactions", post(create_transaction))
        .route("/transactions/:id", get(get_transaction_by_id))
        .route("/transactions/filter", get(filter_transactions))
        .route("/get-block/:chainid/:block_number", get(get_block))
        .route(
            "/get-transaction/:chainid/:block_number/:transaction_hash",
            get(get_transaction),
        )
        .route(
            "/get-native-balance/:chainid/:address",
            get(get_native_balance),
        )
        .route(
            "/get-erc20-balance/:chainid/:contract_address/:address",
            get(get_erc20_balance),
        )
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .layer(cors)
        .with_state(app_state.clone());

    let listener = TcpListener::bind(&config.server_url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    // Spawn the web server on a separate task
    let server_task = task::spawn(async move {
        // Create the server for spawning the task
        serve(listener, app).await.unwrap();
    });

    println!("Web server started!");

    // Ensure the responses directory exists
    fs::create_dir_all("responses").await?;

    let mempool_config = config;

    let mempool_task = task::spawn(async move {
        loop {
            let result: Result<(), AppError> = async {
                scan_mempool(&mempool_config, &app_state).await?;
                Ok(())
            }
            .await;
            if let Err(e) = result {
                error!("Error occurred: {:?}", e);
                error!("Reconnecting in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });

    println!("Mempool scanning started!");

    // shutdown signal
    signal::ctrl_c().await?;
    println!("Shutdown signal received, stopping tasks...");

    // abort tasks
    server_task.abort();
    mempool_task.abort();

    println!("Tasks stopped. Shutting down.");
    Ok(())
}
