use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::users,
};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ResetPasswordService {
    WithToken(ResetPasswordServiceWithToken),
    WithPassword(ResetPasswordServiceWithPassword),
}

#[derive(Deserialize)]
pub struct ResetPasswordServiceWithToken {
    pub token:        String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordServiceWithPassword {
    pub old_password: String,
    pub new_password: String,
}

impl ResetPasswordService {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        user: Option<users::Model>,
    ) -> Response {
        match self {
            ResetPasswordService::WithToken(service) => service.reset_password(conn, redis).await,
            ResetPasswordService::WithPassword(service) => service.reset_password(conn, user).await,
        }
    }
}

impl ResetPasswordServiceWithToken {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
    ) -> Response {
        if self.token.is_empty() || self.new_password.is_empty() {
            return ResponseCode::ParamError.into();
        }

        let key = format!("password-reset::{}", self.token);
        let uid: Option<i32> = match redis::cmd("GETDEL").arg(&key).query_async(redis).await {
            Ok(uid) => uid,
            Err(err) => {
                tracing::error!("Error consuming reset token: {err}");
                return ResponseCode::InternalError.into();
            },
        };

        let Some(uid) = uid else {
            return ResponseCode::Unauthorized.into();
        };

        let Ok((user, _, _)) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::Unauthorized.into();
        };

        if user.status.is_deleted() {
            return ResponseCode::UserDeleted.into();
        }

        if user.check_password(self.new_password.clone()) {
            return ResponseCode::DuplicatePassword.into();
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            tracing::error!("Error updating password by token: {err}");
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}

impl ResetPasswordServiceWithPassword {
    pub async fn reset_password(
        &self,
        conn: &DatabaseConnection,
        user: Option<users::Model>,
    ) -> Response {
        let Some(user) = user else {
            return ResponseCode::Unauthorized.into();
        };

        if !user.check_password(self.old_password.clone()) {
            return ResponseCode::Unauthorized.into();
        }

        if self.old_password == self.new_password {
            return ResponseCode::DuplicatePassword.into();
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            tracing::error!("Error updating password: {err}");
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
