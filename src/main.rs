// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod chunk_pool;
mod config;
mod generator;
mod handlers;
mod streaming;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use config::Config;
use handlers::{garble_handler, health_handler, stats_handler};

/// Wait for a shutdown signal (SIGTERM or SIGINT)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::load_from_file("config.json")?;
    tracing::info!("Loaded configuration: {:?}", config);

    // Create shared state
    let shared_config = Arc::new(config.clone());

    // Start background chunk generation task (this will initialize the pool lazily)
    tracing::info!("Starting background chunk generation task...");
    let background_task = tokio::spawn(async move {
        tracing::info!("Background chunk generation task started");
        let chunk_pool = chunk_pool::CHUNK_POOL.clone();
        chunk_pool.background_maintenance().await;
    });

    // Build the application with routes
    let app = Router::new()
        .route("/garble", get(garble_handler))
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(shared_config);

    // Start the server
    let bind_address = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server on {}", bind_address);

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    tracing::info!("Daddle service is running!");
    tracing::info!("Available endpoints:");
    tracing::info!(
        "  GET /garble - Generate random JSON payload (with smart performance optimization)"
    );
    tracing::info!("  GET /health - Health check endpoint");
    tracing::info!("  GET /stats  - Chunk pool and performance statistics");
    tracing::info!("");
    tracing::info!("Performance features:");
    tracing::info!("  - Chunk pool for fast responses");
    tracing::info!("  - Streaming for large payloads (>1MB)");
    tracing::info!("  - Parallel generation for medium payloads");
    tracing::info!("  - Background chunk generation during idle time");
    tracing::info!("");
    tracing::info!("Example usage:");
    tracing::info!("  curl 'http://{}'/garble", bind_address);
    tracing::info!("  curl 'http://{}'/garble?minBodySize=500&maxBodySize=2000&minWaitDuration=100&maxWaitDuration=500", bind_address);
    tracing::info!("  curl 'http://{}'/garble?minBodySize=8000000&maxBodySize=8000000&minWaitDuration=20&maxWaitDuration=50  # 8MB in 20-50ms!", bind_address);

    // Start the server with graceful shutdown
    tracing::info!("Server starting with graceful shutdown support...");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server has shut down gracefully, stopping background tasks...");

    // Abort the background task since it runs in an infinite loop
    background_task.abort();

    // Wait a moment for the task to clean up
    match tokio::time::timeout(std::time::Duration::from_secs(5), background_task).await {
        Ok(Ok(())) => tracing::info!("Background task completed gracefully"),
        Ok(Err(e)) if e.is_cancelled() => tracing::info!("Background task was cancelled"),
        Ok(Err(e)) => tracing::warn!("Background task error: {}", e),
        Err(_) => tracing::warn!("Background task did not complete within timeout"),
    }

    tracing::info!("All tasks completed, application shutdown complete");
    Ok(())
}
