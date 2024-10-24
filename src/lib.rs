mod routers;
mod state;
mod services;
mod models;
mod internal;

use axum::{Extension, Router};
use colored::Colorize;
use std::{fs, io};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::internal::config::structure::Config;
use crate::models::init::init as init_db;
use crate::routers::get_router;

// Log
use crate::internal::log::layer::LogLayer;
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_logger() {
    tracing_subscriber::registry().with(LogLayer).init();
}

pub async fn run() {
    init_logger();

    let mut config = Config::from_file("config.toml")
        .unwrap_or_else(|_| {
            warn!(message = "Cannot load config file. Use default config instead. ");
            Config::default()
        });

    config.check();
    fs::write("./config.toml", toml::to_string_pretty(&config).unwrap()).unwrap(); // TODO: error handling

    init_db(&config.database.unwrap()).await.unwrap();

    let host = config.host.unwrap();

    let app = Router::new();
    // .layer(Extension(state::AppState {}));
    let listener = TcpListener::bind(format!("{}:{}", &host, config.port.unwrap()))
        .await.unwrap();

    let (tx, rx) = oneshot::channel::<io::Error>();
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, get_router(app)).await {
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