use axum::extract::{FromRequest, Request, rejection::JsonRejection};
use serde::de::DeserializeOwned;

use crate::internal::serializer::{Response, ResponseCode};

#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = Response<String>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let data = axum::Json::<T>::from_request(req, state).await;
        match data {
            Ok(data) => {
                let axum::Json(data) = data;
                Ok(Json(data))
            }
            Err(rejection) => {
                let err = rejection.body_text();

                Err(Response::new(
                    ResponseCode::ParamError.into(),
                    ResponseCode::ParamError.into(),
                    Some(err),
                ))
            }
        }
    }
}
