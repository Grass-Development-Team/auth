use axum::extract::State;
use axum_extra::extract::CookieJar;
use token::services::SessionService;

use crate::{
    infra::error::{AppError, AppErrorKind},
    routers::{
        extractor::LoginAccess,
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
        utils::cookie::CookieJarExt,
    },
    state::AppState,
};

pub async fn controller(
    login: LoginAccess,
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Response<String>) {
    let mut redis = match state.redis.get_multiplexed_tokio_connection().await {
        Ok(redis) => redis,
        Err(err) => {
            return (
                jar,
                app_error_to_response(
                    AppError::infra(
                        AppErrorKind::InternalError,
                        "auth.controller.logout.redis",
                        err,
                    )
                    .with_detail("Unable to connect to redis"),
                ),
            );
        },
    };

    let session = login.session;

    if let Err(err) = SessionService::delete(&mut redis, &session).await {
        return (
            jar,
            app_error_to_response(AppError::infra(
                AppErrorKind::InternalError,
                "auth.controller.logout.delete_session",
                err,
            )),
        );
    }

    let jar = jar.remove_session_cookie();

    (
        jar,
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), None),
    )
}
