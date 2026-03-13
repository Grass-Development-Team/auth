use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Response<T = ()> {
    pub code:    u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:    Option<T>,
}

impl<T> Response<T> {
    pub fn new(code: u16, message: String, data: Option<T>) -> Self {
        Response {
            code,
            message,
            data,
        }
    }
}

impl From<Response> for Value {
    fn from(value: Response) -> Self {
        serde_json::to_value(value).unwrap()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ResponseCode {
    // Http status code
    OK,            // 200
    BadRequest,    // 400
    Unauthorized,  // 401
    Forbidden,     // 403
    NotFound,      // 404
    InternalError, // 500
    // Internal status code
    ParamError,           // 4000
    RegistrationDisabled, // 4001
    MailServiceDisabled,  // 4002
    UserNotFound,         // 4010
    CredentialInvalid,    // 4011
    UserBlocked,          // 4012
    UserNotActivated,     // 4013
    UserExists,           // 4014
    AlreadyLoggedIn,      // 4015
    EmailExists,          // 4016
    UserDeleted,          // 4017
    DuplicatePassword,    // 4018
}

impl ResponseCode {
    pub fn http_status(self) -> StatusCode {
        match self {
            ResponseCode::OK => StatusCode::OK,
            ResponseCode::BadRequest => StatusCode::BAD_REQUEST,
            ResponseCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ResponseCode::Forbidden => StatusCode::FORBIDDEN,
            ResponseCode::NotFound => StatusCode::NOT_FOUND,
            ResponseCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseCode::ParamError => StatusCode::BAD_REQUEST,
            ResponseCode::RegistrationDisabled => StatusCode::FORBIDDEN,
            ResponseCode::MailServiceDisabled => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseCode::UserNotFound => StatusCode::NOT_FOUND,
            ResponseCode::CredentialInvalid => StatusCode::UNAUTHORIZED,
            ResponseCode::UserBlocked => StatusCode::FORBIDDEN,
            ResponseCode::UserNotActivated => StatusCode::FORBIDDEN,
            ResponseCode::UserExists => StatusCode::CONFLICT,
            ResponseCode::AlreadyLoggedIn => StatusCode::CONFLICT,
            ResponseCode::EmailExists => StatusCode::CONFLICT,
            ResponseCode::UserDeleted => StatusCode::NOT_FOUND,
            ResponseCode::DuplicatePassword => StatusCode::CONFLICT,
        }
    }
}

fn status_from_code(code: u16) -> StatusCode {
    match code {
        4000 => StatusCode::BAD_REQUEST,
        4001 => StatusCode::FORBIDDEN,
        4002 => StatusCode::INTERNAL_SERVER_ERROR,
        4010 => StatusCode::NOT_FOUND,
        4011 => StatusCode::UNAUTHORIZED,
        4012 => StatusCode::FORBIDDEN,
        4013 => StatusCode::FORBIDDEN,
        4014 => StatusCode::CONFLICT,
        4015 => StatusCode::CONFLICT,
        4016 => StatusCode::CONFLICT,
        4017 => StatusCode::NOT_FOUND,
        4018 => StatusCode::CONFLICT,
        _ => StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Error code
impl From<ResponseCode> for u16 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => 200,
            ResponseCode::BadRequest => 400,
            ResponseCode::Unauthorized => 401,
            ResponseCode::Forbidden => 403,
            ResponseCode::NotFound => 404,
            ResponseCode::InternalError => 500,
            ResponseCode::ParamError => 4000,
            ResponseCode::RegistrationDisabled => 4001,
            ResponseCode::MailServiceDisabled => 4002,
            ResponseCode::UserNotFound => 4010,
            ResponseCode::CredentialInvalid => 4011,
            ResponseCode::UserBlocked => 4012,
            ResponseCode::UserNotActivated => 4013,
            ResponseCode::UserExists => 4014,
            ResponseCode::AlreadyLoggedIn => 4015,
            ResponseCode::EmailExists => 4016,
            ResponseCode::UserDeleted => 4017,
            ResponseCode::DuplicatePassword => 4018,
        }
    }
}

// Error message
impl From<ResponseCode> for String {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => "OK".into(),
            ResponseCode::BadRequest => "Bad Request".into(),
            ResponseCode::Unauthorized => "Unauthorized".into(),
            ResponseCode::Forbidden => "Forbidden".into(),
            ResponseCode::NotFound => "Not Found".into(),
            ResponseCode::InternalError => "Internal Error".into(),
            ResponseCode::ParamError => "Parameter Error".into(),
            ResponseCode::RegistrationDisabled => "Registration is disabled".into(),
            ResponseCode::MailServiceDisabled => "Mail service is disabled".into(),
            ResponseCode::UserNotFound => "Cannot found user".into(),
            ResponseCode::CredentialInvalid => "Invalid credential".into(),
            ResponseCode::UserBlocked => "The account was blocked".into(),
            ResponseCode::UserNotActivated => "The account is not activated".into(),
            ResponseCode::UserExists => "User already exists".into(),
            ResponseCode::AlreadyLoggedIn => "The account is already logged in".into(),
            ResponseCode::EmailExists => "Email already exists".into(),
            ResponseCode::UserDeleted => "User has been deleted".into(),
            ResponseCode::DuplicatePassword => "Duplicate passwords are not allowed.".into(),
        }
    }
}

impl<T> From<ResponseCode> for Response<T> {
    fn from(value: ResponseCode) -> Self {
        Response::<T>::new_error(value.into(), value.into())
    }
}

impl<T> From<ResponseCode> for Json<Response<T>> {
    fn from(value: ResponseCode) -> Self {
        Json::from(Response::<T>::from(value))
    }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let status = status_from_code(self.code);
        let res = Json(self);

        (status, res).into_response()
    }
}

impl IntoResponse for ResponseCode {
    fn into_response(self) -> axum::response::Response {
        (self.http_status(), Json::<Response>::from(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_code_http_status_mapping_is_correct() {
        assert_eq!(ResponseCode::OK.http_status(), StatusCode::OK);
        assert_eq!(
            ResponseCode::Unauthorized.http_status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ResponseCode::ParamError.http_status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ResponseCode::UserNotFound.http_status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ResponseCode::EmailExists.http_status(),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn response_code_into_response_uses_http_status_mapping() {
        assert_eq!(
            ResponseCode::CredentialInvalid.into_response().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ResponseCode::RegistrationDisabled.into_response().status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ResponseCode::DuplicatePassword.into_response().status(),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn response_into_response_maps_business_code_to_http_status() {
        let conflict = Response::<()>::new(4016, "Email already exists".into(), None);
        assert_eq!(conflict.into_response().status(), StatusCode::CONFLICT);

        let not_found = Response::<()>::new(4010, "Cannot found user".into(), None);
        assert_eq!(not_found.into_response().status(), StatusCode::NOT_FOUND);

        let unauthorized = Response::<()>::new(4011, "Invalid credential".into(), None);
        assert_eq!(
            unauthorized.into_response().status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn response_into_response_keeps_http_code_and_handles_unknown_code() {
        let plain_http = Response::<()>::new(418, "teapot".into(), None);
        assert_eq!(plain_http.into_response().status(), StatusCode::IM_A_TEAPOT);

        let unknown = Response::<()>::new(4999, "unknown".into(), None);
        assert_eq!(
            unknown.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
