use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    internal::serializer::{Response, ResponseCode},
    models::users,
};

#[derive(Deserialize)]
pub struct ResetPasswordService {
    pub old_password: String,
    pub new_password: String,
}

impl ResetPasswordService {
    pub async fn reset_password(&self, conn: &DatabaseConnection, user: users::Model) -> Response {
        if !user.check_password(self.old_password.clone()) {
            return ResponseCode::Unauthorized.into();
        }

        if self.old_password == self.new_password {
            return ResponseCode::DuplicatePassword.into();
        }

        if let Err(err) = user.update_password(conn, self.new_password.clone()).await {
            tracing::error!("Error updating password: {err}");
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
