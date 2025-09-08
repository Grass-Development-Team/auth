use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::{permission, role, users},
};

/// Service handling user delete operations
#[derive(Deserialize, Serialize)]
pub struct DeleteService {
    pub password: String,
}

impl DeleteService {
    pub async fn delete(&self, conn: &DatabaseConnection, user: users::Model) -> Response {
        if user.check_permission(conn, "user:undeletable").await {
            return ResponseCode::Forbidden.into();
        }

        if !user.check_password(self.password.clone()) {
            return Response::new_error(
                ResponseCode::CredentialInvalid.into(),
                "Wrong password".into(),
            );
        }

        if users::delete_user(conn, user.uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}

pub struct AdminDeleteService;

impl AdminDeleteService {
    pub async fn delete(&self, conn: &DatabaseConnection, uid: i32, op_level: i32) -> Response {
        let Ok(user_status) = users::get_user_status(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        if user_status.is_deleted() {
            return ResponseCode::UserDeleted.into();
        }

        if permission::check_permission(conn, uid, "user:undeletable").await {
            return ResponseCode::Forbidden.into();
        }

        let Ok(level) = role::get_user_role_level(conn, uid).await else {
            return ResponseCode::InternalError.into();
        };

        if op_level < level {
            return ResponseCode::Forbidden.into();
        }

        if users::delete_user(conn, uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
