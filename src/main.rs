mod init;

mod internal;
mod models;
mod routers;
mod services;
mod state;

use anyhow::Ok;
use axum::{Extension, Router};
use colored::Colorize;
use std::io;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::oneshot;
use tracing::{info, warn};

use crate::internal::config::Config;
use crate::routers::get_router;

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
            info!("Shutting down...");
        },
        _ = terminate => {
            info!("Shutting down...");
        },
    }
}

/// Entrypoint of the application
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init::logger();
    let mut config = Config::from_file("config.toml").unwrap_or_else(|_| {
        warn!(message = "Cannot load config file. Use default config instead.");
        Config::default()
    });

    config.check();
    config.write("config.toml")?;

    let host = config.host.clone();

    // Initialize database & redis
    let db = init::db(&config.database.clone()).await.unwrap();
    let redis = init::redis(config.redis.clone());

    let app = get_router(Router::new()).with_state(state::AppState {
        db: Arc::from(db),
        redis: Arc::from(redis),
        config: config.clone(),
    });

    let listener = TcpListener::bind(format!("{}:{}", &host, config.port))
        .await
        .unwrap();

    // Start server
    let (tx, rx) = oneshot::channel::<io::Error>();
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
        {
            tx.send(err).unwrap();
        }
    });

    info!(
        "Server started on {}",
        format!("{}:{}", &host, config.port).green()
    );
    let _ = rx.await;
    info!("Server stopped");

    Ok(())
}
