use crate::infra::http::serializer::{Response, ResponseCode};

pub async fn controller() -> Response {
    ResponseCode::NotFound.into()
}
