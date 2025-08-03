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
    pub async fn delete(&self, conn: &DatabaseConnection, uid: i32) -> Response {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        if permission::check_permission(conn, uid, "user:undeletable").await {
            return ResponseCode::Forbidden.into();
        }

        if !user.0.check_password(self.password.clone()) {
            return Response::new_error(
                ResponseCode::CredentialInvalid.into(),
                "Wrong password".into(),
            );
        }

        if users::delete_user(conn, uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}

pub struct AdminDeleteService;

impl AdminDeleteService {
    pub async fn delete(&self, conn: &DatabaseConnection, uid: i32) -> Response {
        if permission::check_permission(conn, uid, "user:undeletable").await {
            return ResponseCode::Forbidden.into();
        }

        let Ok(level) = role::get_user_role_level(conn, uid).await else {
            return ResponseCode::InternalError.into();
        };

        // TODO: Check operator level

        if users::delete_user(conn, uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
