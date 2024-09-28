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
    OK, // 200
    BadRequest, // 400
    Unauthorized, // 401
    NotFound, // 404
}

impl From<ResponseCode> for u16 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::OK => 200,
            ResponseCode::BadRequest => 400,
            ResponseCode::Unauthorized => 401,
            ResponseCode::NotFound => 404,
        }
    }
}