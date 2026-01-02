use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};

use crate::{
    internal::{
        serializer::{Response, ResponseCode},
        validator::Validatable,
    },
    models::{
        role,
        user_info::{self, Gender},
        users,
    },
};

#[derive(Deserialize, Serialize)]
pub struct UpdateService {
    // pub email: Option<String>,
    // pub username: Option<String>,
    pub nickname: Option<String>,
    // pub status: Option<AccountStatus>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub gender: Option<Gender>,
}

impl UpdateService {
    pub async fn update(
        &self,
        conn: &DatabaseConnection,
        user: users::Model,
        info: user_info::Model,
    ) -> Response {
        if user.status.is_inactive() {
            return ResponseCode::UserNotActivated.into();
        }

        if user.status.is_banned() {
            return ResponseCode::UserBlocked.into();
        }

        if let Err(err) = self.validate() {
            return err;
        }

        let res: anyhow::Result<()> = async {
            let mut user = user.into_active_model();
            // if let Some(email) = self.email.clone() {
            //     user.email = Set(email);
            // }
            // if let Some(username) = &self.username {
            //     user.username = Set(username.clone());
            // }
            if let Some(nickname) = &self.nickname {
                user.nickname = Set(nickname.clone());
            }
            // if let Some(status) = &self.status {
            //     user.status = Set(status.clone());
            // }
            user.update(conn).await?;

            let mut info = info.into_active_model();
            if let Some(avatar) = &self.avatar {
                info.avatar = Set(if avatar.is_empty() {
                    None
                } else {
                    Some(avatar.clone())
                });
            }
            if let Some(description) = &self.description {
                info.description = Set(if description.is_empty() {
                    None
                } else {
                    Some(description.clone())
                });
            }
            if let Some(state) = &self.state {
                info.state = Set(if state.is_empty() {
                    None
                } else {
                    Some(state.clone())
                });
            }
            if let Some(gender) = &self.gender {
                info.gender = Set(Some(gender.clone()));
            }
            info.update(conn).await?;

            Ok(())
        }
        .await;

        match res {
            Ok(_) => ResponseCode::OK.into(),
            Err(err) => {
                tracing::error!("Error updating user: {err}");
                ResponseCode::InternalError.into()
            }
        }
    }

    pub async fn update_by_uid(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op_level: i32,
    ) -> Response {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        let Ok(level) = role::get_user_role_level(conn, uid).await else {
            return ResponseCode::InternalError.into();
        };

        if level >= op_level {
            return ResponseCode::Forbidden.into();
        }

        self.update(conn, user.0, user.1).await
    }
}

impl Validatable<Response> for UpdateService {
    fn validate(&self) -> Result<(), Response> {
        if let Some(nickname) = &self.nickname
            && nickname.len() < 3
        {
            return Err(Response::new(
                ResponseCode::ParamError.into(),
                "Nickname should be at least 3 characters long".into(),
                None,
            ));
        }

        Ok(())
    }
}
