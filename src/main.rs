use axum::{
    routing::{get, post},
    Router, Server,
};
use dotenv::dotenv;
use log::error;
use sqlx::postgres::PgPoolOptions;
use std::{error::Error, net::TcpListener, sync::Arc};
use tokio::{
    fs::{self},
    signal, task,
    time::Duration,
};

use sentinel::{
    connection::load_config,
    mempool::mempool::scan_mempool,
    primitive::{AppError, AppState},
    service::{create_transaction, filter_transactions, get_transaction_by_id, get_transactions},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    
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

    let app = Router::new()
        .route("/", get(|| async { "Hello World!!!!" }))
        .route("/transactions", get(get_transactions))
        .route("/transactions", post(create_transaction))
        .route("/transactions/:id", get(get_transaction_by_id))
        .route("/transactions/filter", get(filter_transactions))
        .with_state(app_state.clone());

    let listener = TcpListener::bind(&config.server_url).expect("Could not create TCP Listener");
    println!("listening on {}", listener.local_addr().unwrap());

    // Create the server before spawning the task
    let server = Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service());

    // Spawn the web server on a separate task
    let server_task = task::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
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
