use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Response<T = ()> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
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

#[derive(Serialize, Deserialize)]
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
        match value {
            ResponseCode::OK => {
                Response::<T>::new_error(ResponseCode::OK.into(), ResponseCode::OK.into())
            }
            ResponseCode::BadRequest => Response::<T>::new_error(
                ResponseCode::BadRequest.into(),
                ResponseCode::BadRequest.into(),
            ),
            ResponseCode::Unauthorized => Response::<T>::new_error(
                ResponseCode::Unauthorized.into(),
                ResponseCode::Unauthorized.into(),
            ),
            ResponseCode::Forbidden => Response::<T>::new_error(
                ResponseCode::Forbidden.into(),
                ResponseCode::Forbidden.into(),
            ),
            ResponseCode::NotFound => Response::<T>::new_error(
                ResponseCode::NotFound.into(),
                ResponseCode::NotFound.into(),
            ),
            ResponseCode::InternalError => Response::<T>::new_error(
                ResponseCode::InternalError.into(),
                ResponseCode::InternalError.into(),
            ),
            ResponseCode::ParamError => Response::<T>::new_error(
                ResponseCode::ParamError.into(),
                ResponseCode::ParamError.into(),
            ),
            ResponseCode::RegistrationDisabled => Response::<T>::new_error(
                ResponseCode::RegistrationDisabled.into(),
                ResponseCode::RegistrationDisabled.into(),
            ),
            ResponseCode::UserNotFound => Response::<T>::new_error(
                ResponseCode::UserNotFound.into(),
                ResponseCode::UserNotFound.into(),
            ),
            ResponseCode::CredentialInvalid => Response::<T>::new_error(
                ResponseCode::CredentialInvalid.into(),
                ResponseCode::CredentialInvalid.into(),
            ),
            ResponseCode::UserBlocked => Response::<T>::new_error(
                ResponseCode::UserBlocked.into(),
                ResponseCode::UserBlocked.into(),
            ),
            ResponseCode::UserNotActivated => Response::<T>::new_error(
                ResponseCode::UserNotActivated.into(),
                ResponseCode::UserNotActivated.into(),
            ),
            ResponseCode::UserExists => Response::<T>::new_error(
                ResponseCode::UserExists.into(),
                ResponseCode::UserExists.into(),
            ),
            ResponseCode::AlreadyLoggedIn => Response::<T>::new_error(
                ResponseCode::AlreadyLoggedIn.into(),
                ResponseCode::AlreadyLoggedIn.into(),
            ),
            ResponseCode::EmailExists => Response::<T>::new_error(
                ResponseCode::EmailExists.into(),
                ResponseCode::EmailExists.into(),
            ),
            ResponseCode::UserDeleted => Response::<T>::new_error(
                ResponseCode::UserDeleted.into(),
                ResponseCode::UserDeleted.into(),
            ),
            ResponseCode::DuplicatePassword => Response::<T>::new_error(
                ResponseCode::DuplicatePassword.into(),
                ResponseCode::DuplicatePassword.into(),
            ),
        }
    }
}

impl<T> From<ResponseCode> for Json<Response<T>> {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => Json::from(Response::<T>::new_error(
                ResponseCode::OK.into(),
                ResponseCode::OK.into(),
            )),
            ResponseCode::BadRequest => Json::from(Response::<T>::new_error(
                ResponseCode::BadRequest.into(),
                ResponseCode::BadRequest.into(),
            )),
            ResponseCode::Unauthorized => Json::from(Response::<T>::new_error(
                ResponseCode::Unauthorized.into(),
                ResponseCode::Unauthorized.into(),
            )),
            ResponseCode::Forbidden => Json::from(Response::<T>::new_error(
                ResponseCode::Forbidden.into(),
                ResponseCode::Forbidden.into(),
            )),
            ResponseCode::NotFound => Json::from(Response::<T>::new_error(
                ResponseCode::NotFound.into(),
                ResponseCode::NotFound.into(),
            )),
            ResponseCode::InternalError => Json::from(Response::<T>::new_error(
                ResponseCode::InternalError.into(),
                ResponseCode::InternalError.into(),
            )),
            ResponseCode::ParamError => Json::from(Response::<T>::new_error(
                ResponseCode::ParamError.into(),
                ResponseCode::ParamError.into(),
            )),
            ResponseCode::RegistrationDisabled => Json::from(Response::<T>::new_error(
                ResponseCode::RegistrationDisabled.into(),
                ResponseCode::RegistrationDisabled.into(),
            )),
            ResponseCode::UserNotFound => Json::from(Response::<T>::new_error(
                ResponseCode::UserNotFound.into(),
                ResponseCode::UserNotFound.into(),
            )),
            ResponseCode::CredentialInvalid => Json::from(Response::<T>::new_error(
                ResponseCode::CredentialInvalid.into(),
                ResponseCode::CredentialInvalid.into(),
            )),
            ResponseCode::UserBlocked => Json::from(Response::<T>::new_error(
                ResponseCode::UserBlocked.into(),
                ResponseCode::UserBlocked.into(),
            )),
            ResponseCode::UserNotActivated => Json::from(Response::<T>::new_error(
                ResponseCode::UserNotActivated.into(),
                ResponseCode::UserNotActivated.into(),
            )),
            ResponseCode::UserExists => Json::from(Response::<T>::new_error(
                ResponseCode::UserExists.into(),
                ResponseCode::UserExists.into(),
            )),
            ResponseCode::AlreadyLoggedIn => Json::from(Response::<T>::new_error(
                ResponseCode::AlreadyLoggedIn.into(),
                ResponseCode::AlreadyLoggedIn.into(),
            )),
            ResponseCode::EmailExists => Json::from(Response::<T>::new_error(
                ResponseCode::EmailExists.into(),
                ResponseCode::EmailExists.into(),
            )),
            ResponseCode::UserDeleted => Json::from(Response::<T>::new_error(
                ResponseCode::UserDeleted.into(),
                ResponseCode::UserDeleted.into(),
            )),
            ResponseCode::DuplicatePassword => Json::from(Response::<T>::new_error(
                ResponseCode::DuplicatePassword.into(),
                ResponseCode::DuplicatePassword.into(),
            )),
        }
    }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let status = if self.code < 1000 { self.code } else { 200 };
        let res = Json(self);

        (StatusCode::from_u16(status).unwrap(), res).into_response()
    }
}

impl IntoResponse for ResponseCode {
    fn into_response(self) -> axum::response::Response {
        let (status, res): (u16, Json<Response>) = match self {
            ResponseCode::OK => (ResponseCode::OK.into(), ResponseCode::OK.into()),
            ResponseCode::BadRequest => (
                ResponseCode::BadRequest.into(),
                ResponseCode::BadRequest.into(),
            ),
            ResponseCode::Unauthorized => (
                ResponseCode::Unauthorized.into(),
                ResponseCode::Unauthorized.into(),
            ),
            ResponseCode::Forbidden => (
                ResponseCode::Forbidden.into(),
                ResponseCode::Forbidden.into(),
            ),
            ResponseCode::NotFound => {
                (ResponseCode::NotFound.into(), ResponseCode::NotFound.into())
            }
            ResponseCode::InternalError => (
                ResponseCode::InternalError.into(),
                ResponseCode::InternalError.into(),
            ),
            ResponseCode::ParamError => (ResponseCode::OK.into(), ResponseCode::ParamError.into()),
            ResponseCode::RegistrationDisabled => (
                ResponseCode::OK.into(),
                ResponseCode::RegistrationDisabled.into(),
            ),
            ResponseCode::UserNotFound => {
                (ResponseCode::OK.into(), ResponseCode::UserNotFound.into())
            }
            ResponseCode::CredentialInvalid => (
                ResponseCode::OK.into(),
                ResponseCode::CredentialInvalid.into(),
            ),
            ResponseCode::UserBlocked => {
                (ResponseCode::OK.into(), ResponseCode::UserBlocked.into())
            }
            ResponseCode::UserNotActivated => (
                ResponseCode::OK.into(),
                ResponseCode::UserNotActivated.into(),
            ),
            ResponseCode::UserExists => (ResponseCode::OK.into(), ResponseCode::UserExists.into()),
            ResponseCode::AlreadyLoggedIn => (
                ResponseCode::OK.into(),
                ResponseCode::AlreadyLoggedIn.into(),
            ),
            ResponseCode::EmailExists => {
                (ResponseCode::OK.into(), ResponseCode::EmailExists.into())
            }
            ResponseCode::UserDeleted => {
                (ResponseCode::OK.into(), ResponseCode::UserDeleted.into())
            }
            ResponseCode::DuplicatePassword => (
                ResponseCode::OK.into(),
                ResponseCode::DuplicatePassword.into(),
            ),
        };
        (StatusCode::from_u16(status).unwrap(), res).into_response()
    }
}
