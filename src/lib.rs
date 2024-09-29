mod routers;
mod internal;

use crate::internal::config::structure::Config;
use crate::routers::get_router;
use axum::Router;
use log::{error, info, warn};
use tokio::net::TcpListener;

pub async fn run() {
    let mut config = Config::from_file("config.toml")
        .unwrap_or_else(|_| {
            warn!("Cannot load config file. Use default config instead.");
            Config::default()
        });

    config.check();

    let host = config.host.unwrap();

    let app = Router::new();
    let listener = TcpListener::bind(format!("{}:{}", &host, config.port.unwrap()))
        .await.unwrap();
    match axum::serve(listener, get_router(app)).await {
        Ok(_) => {
            info!("Server started on {}:{}", &host, config.port.unwrap());
        }
        Err(err) => {
            error!("{}", err);
        }
    }
}