// Why fucking shit has this rule??
#![allow(clippy::upper_case_acronyms)]

mod init;

mod domain;
mod features;
mod infra;
mod state;

use std::{io, sync::Arc};

use axum::Router;
use colored::Colorize;
use tokio::{net::TcpListener, signal, sync::oneshot};
use tracing::{error, info};

use crate::infra::config::Config;

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
    let Ok(mut config) = Config::from_file("config.toml") else {
        error!("Cannot load config file. Generating default config instead.");
        return Config::default().write("config.toml");
    };

    config.check();
    config.write("config.toml")?;

    let host = config.host.clone();

    info!("Initializing...");

    // Initialize
    let db = init::db(&config.database.clone()).await?;
    info!("Database initialized.");

    let redis = init::redis(&config.redis)?;
    info!("Redis initialized.");

    let mail = if let Some(mail) = &config.mail {
        Some(Arc::new(init::mailer(mail)?))
    } else {
        None
    };
    if mail.is_some() {
        info!("Mail initialized.");
    }

    state::APP_STATE
        .get_or_init(async || state::AppState {
            db: Arc::from(db),
            redis: Arc::from(redis),
            config: Arc::from(config.clone()),
            mail,
        })
        .await;

    let app = features::router(Router::new(), &config)
        .with_state(state::APP_STATE.get().unwrap().clone());

    let addr = format!("{}:{}", &host, config.port);

    let listener = TcpListener::bind(&addr).await?;

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

    info!("Server started on {}", &addr.green());
    let _ = rx.await;
    info!("Server stopped");

    Ok(())
}
