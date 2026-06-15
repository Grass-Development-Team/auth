use std::env;

use tracing_subscriber::{
    Layer, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::infra::{
    config::{CacheBackend, Config, Redis},
    logger::layer::LogLayer,
};
pub use crate::infra::{database::init as db, mailer::init as mailer};

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

pub fn config() -> anyhow::Result<Config> {
    match Config::from_file("config.toml") {
        Ok(mut config) => {
            config.check();
            config.write("config.toml")?;

            Ok(config)
        },
        Err(err) => {
            Config::default().write("config.toml")?;
            Err(err)
        },
    }
}

fn redis_url(redis: &Redis) -> String {
    format!(
        "redis://{}{}:{}",
        if redis.username.is_some() || redis.password.is_some() {
            format!(
                "{}{}@",
                redis.username.clone().unwrap_or_default(),
                if let Some(password) = redis.password.clone() {
                    format!(":{password}")
                } else {
                    String::new()
                }
            )
        } else {
            String::new()
        },
        redis.host,
        redis.port.unwrap_or(6379)
    )
}

/// Initialize cache backend by config.
pub fn cache(config: &Config) -> Result<cache::Cache, cache::CacheError> {
    match config.cache.backend {
        CacheBackend::Redis => {
            let url = redis_url(&config.redis);
            Ok(cache::Cache::Redis(cache::RedisCache::new(&url)?))
        },
        CacheBackend::Moka => Ok(cache::Cache::Moka(cache::MokaCache::new(
            config.cache.moka.max_capacity,
        ))),
    }
}
