mod internal;
mod models;
mod routers;
mod services;
mod state;

use anyhow::Ok;
use axum::{Extension, Router};
use colored::Colorize;
use std::sync::Arc;
use std::{env, io};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::oneshot;

use crate::internal::config::{Config, Redis};
use crate::models::init as init_db;
use crate::routers::get_router;

// Log
use crate::internal::log::layer::LogLayer;
use tracing::{info, warn};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize the logger
fn init_logger() {
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "".into());
    let level = match level.as_str() {
        "TRACE" => LevelFilter::TRACE,
        "DEBUG" => LevelFilter::DEBUG,
        "WARN" => LevelFilter::WARN,
        "ERROR" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };
    tracing_subscriber::registry()
        .with(LogLayer.with_filter(level))
        .init();
}

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

/// Initialize Redis client
fn init_redis(redis: Redis) -> redis::Client {
    redis::Client::open(format!(
        "redis://{}{}:{}",
        if redis.username.is_some() || redis.password.is_some() {
            format!(
                "{}{}@",
                if redis.username.is_some() {
                    redis.username.unwrap()
                } else {
                    "".into()
                },
                if redis.password.is_some() {
                    format!(":{}", redis.password.unwrap())
                } else {
                    "".into()
                }
            )
        } else {
            "".into()
        },
        redis.host,
        if redis.port.is_some() {
            redis.port.unwrap()
        } else {
            6379
        }
    ))
    .unwrap_or_else(|e| panic!("Error connecting to Redis: {}", e))
}

/// Entrypoint of the application
pub async fn run() -> anyhow::Result<()> {
    init_logger();

    let mut config = Config::from_file("config.toml").unwrap_or_else(|_| {
        warn!(message = "Cannot load config file. Use default config instead.");
        Config::default()
    });

    config.check();
    config.write("config.toml")?;

    let host = config.host.clone();

    // Initialize database & redis
    let db = init_db(&config.database.clone()).await.unwrap();
    let redis = init_redis(config.redis.clone())
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let app = get_router(Router::new()).layer(Extension(state::AppState {
        db: Arc::from(db),
        redis,
        config: config.clone(),
    }));

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
