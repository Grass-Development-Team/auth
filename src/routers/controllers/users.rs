use axum::extract::{Path, State};
use axum_extra::extract::CookieJar;

use crate::{
    internal::{
        error::{AppError, AppErrorKind},
        extractor::{Json, LoginAccess, OperatorAccess},
        session::SessionService,
        utils::cookie::CookieJarExt,
    },
    routers::{
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
    },
    services::users,
    state::AppState,
};

/// User info
pub async fn info(
    State(state): State<AppState>,
    login: LoginAccess,
) -> Response<users::InfoResponse> {
    let service = users::InfoService;
    let (user, info, settings) = login.user;

    match service.info(&state.db, user, info, settings, None).await {
        Ok(data) => Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(data)),
        Err(err) => app_error_to_response(err),
    }
}

/// User info by uid
pub async fn info_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response<users::InfoResponse> {
    let service = users::InfoService;

    match service.info_by_uid(&state.db, uid, login.user.0).await {
        Ok(data) => Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(data)),
        Err(err) => app_error_to_response(err),
    }
}

/// User delete
pub async fn delete(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<users::DeleteService>,
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

    let session = login.session;

    if let Err(err) = req.delete(&state.db, login.user.0).await {
        return (jar, app_error_to_response(err));
    }

    if let Err(err) = SessionService::delete(&mut redis, &session).await {
        return (jar, app_error_to_response(err));
    }

    let jar = jar.remove_session_cookie();

    (jar, ResponseCode::OK.into())
}

/// User delete by uid
pub async fn delete_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response {
    let service = users::AdminDeleteService;

    match service.delete(&state.db, uid, login.level).await {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}

/// User update
pub async fn update(
    login: LoginAccess,
    State(state): State<AppState>,
    Json(req): Json<users::UpdateService>,
) -> Response {
    match req.update(&state.db, login.user.0, login.user.1).await {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}

/// User update by uid
pub async fn update_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
    Json(req): Json<users::UpdateService>,
) -> Response {
    match req.update_by_uid(&state.db, uid, login.level).await {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}
