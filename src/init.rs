use std::env;

// Log
use crate::internal::log::layer::LogLayer;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::internal::config::Redis;

pub use crate::models::init as db;

/// Initialize the logger
pub fn logger() {
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

/// Initialize Redis client
pub fn redis(redis: Redis) -> redis::Client {
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
    .unwrap_or_else(|e| panic!("Error connecting to Redis: {e}"))
}
