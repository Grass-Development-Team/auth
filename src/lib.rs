mod routers;
mod state;
mod services;
mod models;
mod internal;

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
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

fn init_logger() {
    let level = env::var("LOG_LEVEL").unwrap_or_else(|_| {
        "".into()
    });
    let level = match level.as_str() {
        "TRACE" => LevelFilter::TRACE,
        "DEBUG" => LevelFilter::DEBUG,
        "WARN" => LevelFilter::WARN,
        "ERROR" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };
    tracing_subscriber::registry().with(LogLayer.with_filter(level)).init();
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

fn init_redis(redis: Redis) -> redis::Client {
    redis::Client::open(
        format!(
            "redis://{}{}:{}",
            if redis.username.is_some() {
                format!("{}{}@", redis.username.unwrap(), if redis.password.is_some() {
                    format!(":{}", redis.password.unwrap())
                } else { "".into() })
            } else { "".into() },
            redis.host,
            if redis.port.is_some() { redis.port.unwrap() } else { 6379 }
        )
    ).unwrap_or_else(|e| { panic!("Error connecting to Redis: {}", e) })
}

pub async fn run() {
    init_logger();

    let mut config = Config::from_file("config.toml")
        .unwrap_or_else(|_| {
            warn!(message = "Cannot load config file. Use default config instead. ");
            Config::default()
        });

    config.check();
    config.write("./config.toml");

    let host = config.host.unwrap();

    let db = init_db(&config.database.unwrap()).await.unwrap();
    let redis = init_redis(config.redis)
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let app = get_router(Router::new())
        .layer(
            Extension(state::AppState {
                db: Arc::from(db),
                redis: Arc::from(redis),
            })
        );

    let listener = TcpListener::bind(format!("{}:{}", &host, config.port.unwrap()))
        .await.unwrap();

    let (tx, rx) = oneshot::channel::<io::Error>();
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await {
            tx.send(err).unwrap();
        }
    });

    info!("Server started on {}", format!("{}:{}", &host, config.port.unwrap()).green());
    let _ = rx.await;
    info!("Server stopped");
}