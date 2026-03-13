use std::env;

use tracing_subscriber::{
    Layer, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::internal::{config::Redis, log::layer::LogLayer};
pub use crate::{internal::mail::init as mail, models::init as db};

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
pub fn redis(redis: &Redis) -> Result<redis::Client, redis::RedisError> {
    redis::Client::open(format!(
        "redis://{}{}:{}",
        if redis.username.is_some() || redis.password.is_some() {
            format!(
                "{}{}@",
                if let Some(username) = redis.username.clone() {
                    username
                } else {
                    "".into()
                },
                if let Some(password) = redis.password.clone() {
                    format!(":{}", password)
                } else {
                    "".into()
                }
            )
        } else {
            "".into()
        },
        redis.host,
        redis.port.unwrap_or(6379)
    ))
}
