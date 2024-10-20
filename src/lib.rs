mod routers;
mod state;
mod services;
mod models;
mod internal;

use axum::{Extension, Router};
use colored::Colorize;
use std::sync::Arc;
use std::{env, fs, io};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::internal::config::structure::Config;
use crate::models::init as init_db;
use crate::routers::get_router;

// Log
use crate::internal::log::layer::LogLayer;
use tracing::{error, info, warn};
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
    let app = Router::new();
    let app = get_router(app)
        .layer(
            Extension(state::AppState {
                db: Arc::from(db)
            })
        );

    let listener = TcpListener::bind(format!("{}:{}", &host, config.port.unwrap()))
        .await.unwrap();

    let (tx, rx) = oneshot::channel::<io::Error>();
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            tx.send(err).unwrap();
        }
    });

    info!("Server started on {}", format!("{}:{}", &host, config.port.unwrap()).green());
    match rx.await {
        Ok(err) => {
            error!("Server stopped with error: {}", err);
        }
        Err(err) => {
            error!("Error while running server: {}", err);
        }
    }
}