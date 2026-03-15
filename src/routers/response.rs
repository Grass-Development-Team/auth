use crate::{
    internal::error::{AppError, AppErrorKind},
    routers::serializer::{Response, ResponseCode},
};

pub fn app_error_to_response<T>(err: AppError) -> Response<T> {
    let code: ResponseCode = err.kind.into();
    let detail = err.detail.clone();

    let infra_kind = matches!(
        err.kind,
        AppErrorKind::InternalError | AppErrorKind::VerificationEmailSendFailed
    );

    if err.source_ref().is_some() || infra_kind {
        let source = err.source_ref().map(ToString::to_string);
        tracing::error!(
            op = err.op,
            kind = ?err.kind,
            detail = ?detail,
            source = ?source,
            "request failed with infrastructure error"
        );
    } else {
        tracing::warn!(
            op = err.op,
            kind = ?err.kind,
            detail = ?detail,
            "request failed with business error"
        );
    }

    let message = detail.unwrap_or_else(|| String::from(code));
    Response::new_error(code.into(), message)
}
