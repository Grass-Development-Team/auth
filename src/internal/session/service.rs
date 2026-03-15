use redis::{AsyncCommands, aio::MultiplexedConnection};
use tracing::{error, trace};

use super::{SESSION_TTL_SECONDS, Session, generate, parse_from_str};
use crate::internal::error::{AppError, AppErrorKind};

/// Session manager backed by Redis.
pub struct SessionService;

/// Result of resolving a session id from Redis.
#[derive(Debug, Clone, Copy)]
pub enum SessionLookup {
    /// No value exists for the requested session key.
    Missing,
    /// Session payload exists but cannot be parsed as [`Session`].
    Invalid,
    /// Session payload is valid but already expired.
    Expired,
    /// Session payload is valid and not expired.
    Valid(Session),
}

impl SessionService {
    /// Create a new session, store it in Redis and return its session id.
    ///
    /// # Arguments
    /// - `redis`: Active Redis multiplexed connection.
    /// - `uid`: User id to bind in the session payload.
    ///
    /// # Returns
    /// - `Ok(session_id)` when session is created and persisted.
    /// - `Err(AppError)` when serialization or Redis write fails.
    pub async fn create(redis: &mut MultiplexedConnection, uid: i32) -> Result<String, AppError> {
        let session = generate(uid);
        trace!("Generate session: {:?}", session);

        let session = serde_json::to_string(&session).map_err(|err| {
            error!("Failed to serialize session: {}", err);
            AppError::infra(AppErrorKind::InternalError, "session.create.serialize", err)
        })?;
        trace!("Session string: {:?}", session);

        let sid = uuid::Uuid::new_v4();
        trace!("Generate session id: {:?}", sid);

        redis
            .set_ex::<_, _, ()>(format!("session::{sid}"), session, SESSION_TTL_SECONDS)
            .await
            .map_err(|err| {
                error!("Redis error: {}", err);
                AppError::infra(
                    AppErrorKind::InternalError,
                    "session.create.redis_set_ex",
                    err,
                )
            })?;

        Ok(sid.to_string())
    }

    /// Delete a session by session id.
    ///
    /// # Arguments
    /// - `redis`: Active Redis multiplexed connection.
    /// - `session_id`: Session id (without the `session::` prefix).
    ///
    /// # Returns
    /// - `Ok(())` when delete operation succeeds.
    /// - `Err(AppError)` when Redis delete fails.
    pub async fn delete(
        redis: &mut MultiplexedConnection,
        session_id: &str,
    ) -> Result<(), AppError> {
        redis
            .del::<_, usize>(format!("session::{session_id}"))
            .await
            .map_err(|err| {
                error!("Failed to delete session {}: {}", session_id, err);
                AppError::infra(AppErrorKind::InternalError, "session.delete.redis_del", err)
            })?;

        Ok(())
    }

    /// Resolve and validate a session id from Redis.
    ///
    /// # Arguments
    /// - `redis`: Active Redis multiplexed connection.
    /// - `session_id`: Session id (without the `session::` prefix).
    ///
    /// # Returns
    /// - `Ok(SessionLookup::Missing)` when key does not exist.
    /// - `Ok(SessionLookup::Invalid)` when payload cannot be parsed.
    /// - `Ok(SessionLookup::Expired)` when payload is parsed but expired.
    /// - `Ok(SessionLookup::Valid(session))` when session is valid.
    /// - `Err(AppError)` when Redis read fails.
    pub async fn resolve(
        redis: &mut MultiplexedConnection,
        session_id: &str,
    ) -> Result<SessionLookup, AppError> {
        let payload = redis
            .get::<_, Option<String>>(format!("session::{session_id}"))
            .await
            .map_err(|err| {
                error!("Failed to fetch session {}: {}", session_id, err);
                AppError::infra(
                    AppErrorKind::InternalError,
                    "session.resolve.redis_get",
                    err,
                )
            })?;

        let Some(payload) = payload else {
            return Ok(SessionLookup::Missing);
        };

        let Some(session) = parse_from_str(&payload) else {
            return Ok(SessionLookup::Invalid);
        };

        if !session.validate() {
            return Ok(SessionLookup::Expired);
        }

        Ok(SessionLookup::Valid(session))
    }
}
