use std::{collections::HashSet, convert::Infallible, pin::Pin, task::Poll};

use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::Cookie;
use redis::AsyncCommands;
use tower::{Layer, Service};

use crate::{
    internal::{serializer, utils},
    models::permission,
    state::APP_STATE,
};

#[derive(Clone)]
enum PermType {
    Any,
    All,
}

#[derive(Clone)]
pub struct PermissionAccessService<S> {
    inner: S,
    perms: &'static [&'static str],
    perm_type: PermType,
}

impl<S> Service<Request<Body>> for PermissionAccessService<S>
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let perms = self.perms;
        let perm_type = self.perm_type.clone();
        let mut inner = self.inner.clone();
        let fut = async move {
            let Some(state) = APP_STATE.get() else {
                return Ok(serializer::ResponseCode::InternalError.into_response());
            };

            let db = state.db.clone();

            let bearer_token = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "));

            if let Some(_token) = bearer_token {
                todo!("Check Token Permission")
            } else {
                let Ok(mut redis) = state.redis.get_multiplexed_tokio_connection().await else {
                    return Ok(serializer::ResponseCode::InternalError.into_response());
                };

                // Get session
                let Some(session) = req
                    .headers()
                    .get("cookie")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .split(';')
                    .find_map(|s| Cookie::parse(s.trim()).ok())
                    .filter(|c| c.name() == "session")
                    .map(|c| c.value().to_string())
                else {
                    return Ok(serializer::ResponseCode::Unauthorized.into_response());
                };

                let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
                    return Ok(serializer::ResponseCode::Unauthorized.into_response());
                };

                let Some(session) = utils::session::parse_from_str(&session) else {
                    return Ok(serializer::ResponseCode::InternalError.into_response());
                };

                let Ok(user_perm) = permission::get_permissions_by_uid(&*db, session.uid).await
                else {
                    return Ok(serializer::ResponseCode::InternalError.into_response());
                };

                match perm_type {
                    PermType::All => {
                        let user_perm: HashSet<&str> =
                            user_perm.iter().map(AsRef::as_ref).collect();
                        if perms.iter().all(|perm| user_perm.contains(perm)) {
                            inner.call(req).await
                        } else {
                            Ok(serializer::ResponseCode::Forbidden.into_response())
                        }
                    }
                    PermType::Any => {
                        let user_perm: HashSet<&str> =
                            user_perm.iter().map(AsRef::as_ref).collect();
                        if perms.iter().any(|perm| user_perm.contains(perm)) {
                            inner.call(req).await
                        } else {
                            Ok(serializer::ResponseCode::Forbidden.into_response())
                        }
                    }
                }
            }
        };
        Box::pin(fut)
    }
}

#[derive(Clone)]
pub struct PermissionAccess {
    perms: &'static [&'static str],
    perm_type: PermType,
}

impl PermissionAccess {
    pub fn all(perms: &'static [&'static str]) -> Self {
        PermissionAccess {
            perms,
            perm_type: PermType::All,
        }
    }

    pub fn any(perms: &'static [&'static str]) -> Self {
        PermissionAccess {
            perms,
            perm_type: PermType::Any,
        }
    }
}

impl<S> Layer<S> for PermissionAccess
where
    S: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Service = PermissionAccessService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PermissionAccessService {
            inner,
            perms: self.perms,
            perm_type: self.perm_type.clone(),
        }
    }
}
