use crate::infra::http::serializer::{Response, ResponseCode};

pub async fn controller() -> Response {
    Response::new(ResponseCode::OK.into(), "pong".into(), None)
}
