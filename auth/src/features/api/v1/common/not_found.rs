use crate::routers::serializer::{Response, ResponseCode};

pub async fn controller() -> Response {
    ResponseCode::NotFound.into()
}
