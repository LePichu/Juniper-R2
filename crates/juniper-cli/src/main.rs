use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path},
};
use clap::{Parser, Subcommand};
use juniper_core::{JuniperRuntime, Runtime};
use std::sync::Arc;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "juniper-cli")]
#[command(about = "Juniper CLI for interacting with the runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the Juniper HTTP server")]
    Server {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            println!("Starting Juniper server on port {}", port);
            
            let mut runtime = JuniperRuntime::new(port);
            if let Err(e) = runtime.initialize().await {
                eprintln!("Failed to initialize runtime: {}", e);
                return;
            }
            
            let runtime_handle = Arc::new(runtime);
            let runtime_clone = runtime_handle.clone();
            
            tokio::spawn(async move {
                if let Err(e) = runtime_clone.start().await {
                    eprintln!("Server error: {}", e);
                }
            });
            
            let app = Router::new()
                .route("/status", get(status_handler))
                .with_state(runtime_handle);
                
            let addr = SocketAddr::from(([127, 0, 0, 1], port + 1));
            println!("Starting CLI HTTP server on {}", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    }
}

async fn status_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}