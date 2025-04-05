use redis::RedisError;
use sea_orm::DbErr;

pub enum ServiceError {
    RedisError(RedisError),
    DBError(DbErr),
    JSONError(serde_json::Error),
    Empty,
}