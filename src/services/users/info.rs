use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::{
        user_info::{self, Gender},
        user_settings, users,
    },
};

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    pub uid:         i32,
    pub status:      &'static str,
    pub username:    String,
    pub nickname:    String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email:       Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state:       Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender:      Option<Gender>,
}

pub struct InfoService;

impl InfoService {
    pub async fn info(
        &self,
        conn: &DatabaseConnection,
        user: users::Model,
        info: user_info::Model,
        settings: user_settings::Model,
        op: Option<users::Model>,
    ) -> Response<InfoResponse> {
        let is_self = op.is_none();
        let read_all_permission = if let Some(op) = &op {
            op.check_permission(conn, "user:read:all").await
        } else {
            false
        };

        if !read_all_permission && user.status.is_deleted() {
            return ResponseCode::UserDeleted.into();
        }

        let show_email = is_self || read_all_permission || settings.show_email;
        let show_gender = is_self || read_all_permission || settings.show_gender;
        let show_state = is_self || read_all_permission || settings.show_state;

        let res = InfoResponse {
            uid:         user.uid,
            status:      user.status.into(),
            username:    user.username,
            email:       if show_email { Some(user.email) } else { None },
            nickname:    user.nickname,
            avatar:      info.avatar,
            description: info.description,
            state:       if show_state { info.state } else { None },
            gender:      if show_gender { info.gender } else { None },
        };

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(res))
    }

    pub async fn info_by_uid(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op: users::Model,
    ) -> Response<InfoResponse> {
        let Ok((user, info, settings)) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        self.info(conn, user, info, settings, Some(op)).await
    }
}
