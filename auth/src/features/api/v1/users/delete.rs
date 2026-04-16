use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use token::services::SessionService;

use crate::{
    domain::{permission, role, users},
    infra::{
        error::{AppError, AppErrorKind},
        http::{
            extractor::{Json, LoginAccess, OperatorAccess},
            response::app_error_to_response,
            serializer::{Response, ResponseCode},
            utils::cookie::CookieJarExt,
        },
    },
    state::AppState,
};

pub async fn controller(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<DeleteService>,
) -> (CookieJar, Response) {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return (
                jar,
                app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "users.controller.delete.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                ),
            );
        },
    };

    let uid = login.user.0.uid;

    if let Err(err) = req.delete(&state.db, login.user.0).await {
        return (jar, app_error_to_response(err));
    }

    if let Err(err) = SessionService::delete_all_by_uid(&mut redis, uid).await {
        return (
            jar,
            app_error_to_response(AppError::infra(
                AppErrorKind::InternalError,
                "users.controller.delete.delete_all_sessions",
                err,
            )),
        );
    }

    let jar = jar.remove_session_cookie();

    (jar, ResponseCode::OK.into())
}

pub async fn controller_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return app_error_to_response(
                AppError::infra(
                    AppErrorKind::InternalError,
                    "users.controller.admin_delete.redis",
                    err,
                )
                .with_detail("Unable to connect to redis"),
            );
        },
    };

    let service = AdminDeleteService;

    match service.delete(&state.db, uid, login.level).await {
        Ok(()) => {
            if let Err(err) = SessionService::delete_all_by_uid(&mut redis, uid).await {
                return app_error_to_response(AppError::infra(
                    AppErrorKind::InternalError,
                    "users.controller.admin_delete.delete_all_sessions",
                    err,
                ));
            }

            ResponseCode::OK.into()
        },
        Err(err) => app_error_to_response(err),
    }
}

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
