use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::{permission, role, users},
};

/// Service handling user delete operations
#[derive(Deserialize, Serialize)]
pub struct DeleteService {
    pub password: Option<String>,
}

impl DeleteService {
    pub async fn delete(&self, conn: &DatabaseConnection, uid: i32, op_uid: i32) -> Response {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        if user.0.status.is_deleted() {
            return ResponseCode::UserDeleted.into();
        }

        if permission::check_permission(conn, uid, "user:undeletable").await {
            return ResponseCode::Forbidden.into();
        }

        if uid == op_uid {
            let Some(password) = self.password.clone() else {
                return ResponseCode::ParamError.into();
            };

            if !user.0.check_password(password) {
                return ResponseCode::CredentialInvalid.into();
            }
        } else {
            let Ok(level) = role::get_user_role_level(conn, uid).await else {
                return ResponseCode::InternalError.into();
            };

            let Ok(op_level) = role::get_user_role_level(conn, op_uid).await else {
                return ResponseCode::InternalError.into();
            };

            if op_level <= level {
                return ResponseCode::Forbidden.into();
            }
        }

        if users::delete_user(conn, uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
