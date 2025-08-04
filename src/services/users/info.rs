use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::{permission, user_info::Gender, users},
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
    pub async fn info(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op_uid: i32,
    ) -> Response<InfoResponse> {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        let info = user.1[0].clone();
        let user = user.0;

        if uid != op_uid && !permission::check_permission(conn, op_uid, "user:read:all").await {
            if user.status.is_deleted() {
                return ResponseCode::UserNotFound.into();
            }

            if user.status.is_inactive() {
                return ResponseCode::UserNotActivated.into();
            }

            if user.status.is_banned() {
                return ResponseCode::UserBlocked.into();
            }
        }

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
}
