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
    NotFound,      // 404
    InternalError, // 500
    // Internal status code
    ParamError,        // 4000
    UserNotFound,      // 4010
    CredentialInvalid, // 4011
    UserBlocked,       // 4012
    UserNotActivated,  // 4013
    UserExists,        // 4014
    AlreadyLoggedIn,   // 4015
    EmailExists,       // 4016
}

// Error code
impl From<ResponseCode> for u16 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => 200,
            ResponseCode::BadRequest => 400,
            ResponseCode::Unauthorized => 401,
            ResponseCode::NotFound => 404,
            ResponseCode::InternalError => 500,
            ResponseCode::ParamError => 4000,
            ResponseCode::UserNotFound => 4010,
            ResponseCode::CredentialInvalid => 4011,
            ResponseCode::UserBlocked => 4012,
            ResponseCode::UserNotActivated => 4013,
            ResponseCode::UserExists => 4014,
            ResponseCode::AlreadyLoggedIn => 4015,
            ResponseCode::EmailExists => 4016,
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
            ResponseCode::NotFound => "Not Found".into(),
            ResponseCode::InternalError => "Internal Error".into(),
            ResponseCode::ParamError => "Parameter Error".into(),
            ResponseCode::UserNotFound => "Cannot found user".into(),
            ResponseCode::CredentialInvalid => "Invalid credential".into(),
            ResponseCode::UserBlocked => "The account was blocked".into(),
            ResponseCode::UserNotActivated => "The account is not activated".into(),
            ResponseCode::UserExists => "User already exists".into(),
            ResponseCode::AlreadyLoggedIn => "The account is already logged in".into(),
            ResponseCode::EmailExists => "Email already exists".into(),
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
        }
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
            ResponseCode::NotFound => {
                (ResponseCode::NotFound.into(), ResponseCode::NotFound.into())
            }
            ResponseCode::InternalError => {
                (ResponseCode::OK.into(), ResponseCode::InternalError.into())
            }
            ResponseCode::ParamError => (ResponseCode::OK.into(), ResponseCode::ParamError.into()),
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
        };
        (StatusCode::from_u16(status).unwrap(), res).into_response()
    }
}
