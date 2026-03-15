use redis::{AsyncCommands, aio::MultiplexedConnection};
use tracing::{error, trace};

use super::{SESSION_TTL_SECONDS, generate};
use crate::internal::error::{AppError, AppErrorKind};

pub struct SessionService;

impl SessionService {
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
}
