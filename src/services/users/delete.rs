use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::serializer::common::{Response, ResponseCode},
    models::users,
};

/// Service handling user delete operations
#[derive(Deserialize, Serialize)]
pub struct DeleteService {
    pub password: String,
}

impl DeleteService {
    pub async fn delete(&self, conn: &DatabaseConnection, uid: i32) -> Response {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return ResponseCode::UserNotFound.into();
        };

        if !user.0.check_password(self.password.clone()) {
            return Response::new_error(
                ResponseCode::CredentialInvalid.into(),
                "Wrong password".into(),
            );
        }

        if users::delete_user(conn, uid).await.is_err() {
            return ResponseCode::InternalError.into();
        }

        ResponseCode::OK.into()
    }
}
