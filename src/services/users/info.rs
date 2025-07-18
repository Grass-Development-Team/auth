use axum_extra::extract::CookieJar;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        serializer::common::{Response, ResponseCode},
        utils,
    },
    models::{user_info::Gender, users},
};

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    pub uid: i32,
    pub username: String,
    pub email: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<Gender>,
}

pub struct InfoService;

impl InfoService {
    pub async fn info(
        &self,
        conn: &DatabaseConnection,
        redis: &mut MultiplexedConnection,
        jar: CookieJar,
    ) -> (CookieJar, Response<InfoResponse>) {
        let Some(session) = jar.get("session") else {
            let jar = jar.remove("session");
            return (jar, ResponseCode::Unauthorized.into());
        };
        let session = session.value();
        let Ok(session) = redis.get::<_, String>(format!("session-{session}")).await else {
            let jar = jar.remove("session");
            return (jar, ResponseCode::Unauthorized.into());
        };

        let Some(session) = utils::session::parse_from_str(&session) else {
            let jar = jar.remove("session");
            return (jar, ResponseCode::Unauthorized.into());
        };

        let Ok(user) = users::get_user_by_id(conn, session.uid).await else {
            let jar = jar.remove("session");
            return (jar, ResponseCode::UserNotFound.into());
        };

        let info = user.1[0].clone();
        let user = user.0;

        let res = InfoResponse {
            uid: user.uid,
            username: user.username,
            email: user.email,
            nickname: user.nickname,
            avatar: info.avatar,
            description: info.description,
            state: info.state,
            gender: info.gender,
        };

        (
            jar,
            Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(res)),
        )
    }
}
