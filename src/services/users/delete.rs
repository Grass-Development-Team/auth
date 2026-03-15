use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::error::{AppError, AppErrorKind},
    models::{permission, role, users},
};

/// Service handling user delete operations
#[derive(Deserialize, Serialize)]
pub struct DeleteService {
    pub password: String,
}

impl DeleteService {
    pub async fn delete(
        &self,
        conn: &DatabaseConnection,
        user: users::Model,
    ) -> Result<(), AppError> {
        if user.check_permission(conn, "user:undeletable").await {
            return Err(AppError::biz(
                AppErrorKind::Forbidden,
                "users.delete.check_undeletable",
            ));
        }

        if !user.check_password(self.password.clone()) {
            return Err(AppError::biz(
                AppErrorKind::CredentialInvalid,
                "users.delete.verify_password",
            )
            .with_detail("Wrong password"));
        }

        if let Err(err) = users::delete_user(conn, user.uid).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "users.delete.persist",
                err,
            ));
        }

        Ok(())
    }
}

pub struct AdminDeleteService;

impl AdminDeleteService {
    pub async fn delete(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op_level: i32,
    ) -> Result<(), AppError> {
        let Ok(user_status) = users::get_user_status(conn, uid).await else {
            return Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "users.admin_delete.find_user_status",
            ));
        };

        if user_status.is_deleted() {
            return Err(AppError::biz(
                AppErrorKind::UserDeleted,
                "users.admin_delete.check_user_status",
            ));
        }

        if permission::check_permission(conn, uid, "user:undeletable").await {
            return Err(AppError::biz(
                AppErrorKind::Forbidden,
                "users.admin_delete.check_undeletable",
            ));
        }

        let level = role::get_user_role_level(conn, uid).await.map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "users.admin_delete.role_level",
                err,
            )
        })?;

        if op_level < level {
            return Err(AppError::biz(
                AppErrorKind::Forbidden,
                "users.admin_delete.check_operator_level",
            ));
        }

        if let Err(err) = users::delete_user(conn, uid).await {
            return Err(AppError::infra(
                AppErrorKind::InternalError,
                "users.admin_delete.persist",
                err,
            ));
        }

        Ok(())
    }
}
