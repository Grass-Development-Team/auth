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
    OK, // 200
    BadRequest, // 400
    Unauthorized, // 401
    NotFound, // 404
    // Internal status code
    ParamError, // 4000
    UserNotFound, // 4010
    CredentialInvalid, // 4011
    UserBlocked, // 4012
    UserNotActivated, // 4013
}

// Error code
impl From<ResponseCode> for u16 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => 200,
            ResponseCode::BadRequest => 400,
            ResponseCode::Unauthorized => 401,
            ResponseCode::NotFound => 404,
            ResponseCode::ParamError => 4000,
            ResponseCode::UserNotFound => 4010,
            ResponseCode::CredentialInvalid => 4011,
            ResponseCode::UserBlocked => 4012,
            ResponseCode::UserNotActivated => 4013,
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
            ResponseCode::ParamError => "".into(),
            ResponseCode::UserNotFound => "Cannot found user".into(),
            ResponseCode::CredentialInvalid => "Invalid credential".into(),
            ResponseCode::UserBlocked => "The account was blocked".into(),
            ResponseCode::UserNotActivated => "The account is not activated".into(),
        }
    }
}