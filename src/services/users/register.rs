use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::models::users::{AccountPermission, AccountStatus};
use crate::models::{user_info, users};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Deserialize, Serialize)]
pub struct RegisterService {
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

impl RegisterService {
    pub async fn register(&self, conn: &DatabaseConnection) -> Response<String> {
        if users::get_user_by_email(conn, self.email.clone())
            .await
            .is_ok()
        {
            return ResponseCode::UserExists.into();
        }

        let salt = utils::rand::string(16);
        let password = utils::password::generate(self.password.to_owned(), salt.to_owned());

        let user = users::ActiveModel {
            username: Set(self.username.to_owned()),
            email: Set(self.email.to_owned()),
            password: Set(format!("sha2:{}:{}", password, salt)),
            nickname: Set(if self.nickname.is_some() {
                self.nickname.to_owned().unwrap()
            } else {
                self.email.split("@").collect::<Vec<&str>>()[0].to_owned()
            }),
            status: Set(AccountStatus::Inactive),
            perm: Set(AccountPermission::User),
            ..Default::default()
        };

        let user = user.insert(conn).await;
        if let Err(err) = user {
            error!("Database Error: {}", err);
            return ResponseCode::InternalError.into();
        }
        let user = user.unwrap();

        info!("{:?}", user);

        let info = user_info::ActiveModel {
            uid: Set(user.uid),
            ..Default::default()
        };

        let info = info.insert(conn).await;
        if let Err(err) = info {
            error!("Database Error:  {}", err);
            return ResponseCode::InternalError.into();
        }

        // TODO: Send Verification Email

        Response::new(
            ResponseCode::OK.into(),
            ResponseCode::OK.into(),
            Some("Register successfully".into()),
        )
    }
}
