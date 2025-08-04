use crate::internal::serializer::{Response, ResponseCode};

pub async fn not_found() -> Response {
    ResponseCode::NotFound.into()
}

pub async fn ping() -> Response {
    Response::new(ResponseCode::OK.into(), "pong".into(), None)
}
