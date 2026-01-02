use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::{
        user_info::{self, Gender},
        users,
    },
};

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    pub uid: i32,
    pub status: &'static str,
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
    pub async fn info(&self, user: users::Model, info: user_info::Model) -> Response<InfoResponse> {
        let res = InfoResponse {
            uid: user.uid,
            status: user.status.into(),
            username: user.username,
            email: user.email,
            nickname: user.nickname,
            avatar: info.avatar,
            description: info.description,
            state: info.state,
            gender: info.gender,
        };

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(res))
    }

    pub async fn info_by_uid(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op: users::Model,
    ) -> Response<InfoResponse> {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        if !op.check_permission(conn, "user:read:all").await && user.0.status.is_deleted() {
            return ResponseCode::UserDeleted.into();
        }

        self.info(user.0, user.1).await
    }
}
