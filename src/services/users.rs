use crate::internal::serializer::common::{Response, ResponseCode};
use crate::internal::utils;
use crate::models::users;
use crate::models::users::{AccountPermission, AccountStatus};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginService {
    pub email: String,
    pub password: String,
}

impl LoginService {
    pub async fn login(&self, conn: &DatabaseConnection) -> Response<&str> {
        let Ok(user) = users::get_user_by_email(conn, self.email.clone()).await else { return Response::new_error(ResponseCode::UserNotFound.into(), ResponseCode::UserNotFound.into()) };

        if !user.check_password(self.password.to_owned()) {
            return Response::new_error(ResponseCode::CredentialInvalid.into(), "Wrong password".into());
        }
        if user.status == AccountStatus::Banned || user.status == AccountStatus::Deleted {
            return Response::new_error(ResponseCode::UserBlocked.into(), ResponseCode::UserBlocked.into());
        }
        if user.status == AccountStatus::Inactive {
            return Response::new_error(ResponseCode::UserNotActivated.into(), ResponseCode::UserNotActivated.into());
        }

        // TODO: 2-factor authentication

        // TODO: Generate Token

        // TODO: Return login info
        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some("Login successfully"))
    }
}

#[derive(Deserialize, Serialize)]
pub struct RegisterService {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

impl RegisterService {
    pub async fn register(&self, conn: &DatabaseConnection) -> Response<&str> {
        if users::get_user_by_email(conn, self.email.clone()).await.is_ok() {
            return Response::new_error(ResponseCode::UserExists.into(), ResponseCode::UserExists.into());
        }

        let salt = utils::rand::string(16);
        let password = utils::password::generate(self.password.to_owned(), salt);

        let mut user = users::ActiveModel::new();
        user.email = Set(self.email.to_owned());
        user.password = Set(password);
        user.nickname = Set(if self.nickname.is_some() { self.nickname.to_owned().unwrap() } else { self.email.split("@").collect::<Vec<&str>>()[0].to_owned() });
        user.status = Set(AccountStatus::Inactive);
        user.perm = Set(AccountPermission::User);

        let user = user.update(conn).await;
        if user.is_err() {
            return Response::new_error(ResponseCode::InternalError.into(), ResponseCode::InternalError.into());
        }

        // TODO: Send Verification Email

        Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some("Register successfully"))
    }
}