use crate::models::common::ModelError;
use crate::models::common::ModelError::{DBError, Empty};

use crate::internal::utils;
use sea_orm::entity::prelude::*;
use sea_orm::QuerySelect;
use serde::{Deserialize, Serialize};

/// Status of the Account
/// - Inactive (0): User haven't active the account through link send to email
/// - Active (1): Active account
/// - Banned (2): Account banned by admin
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum AccountStatus {
    Inactive = 0,
    Active = 1,
    Banned = 2,
    Deleted = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum AccountPermission {
    Root = 0,
    Admin = 1,
    AssistManager = 2,
    User = 3,
    Guest = 4,
}

/// # Users Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub uid: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub status: AccountStatus,
    pub perm: AccountPermission,
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No defined relation")
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Get user model by email
pub async fn get_user_by_email(
    conn: &DatabaseConnection,
    email: String,
) -> Result<Model, ModelError> {
    let res = Entity::find()
        .filter(Column::Email.eq(email))
        .limit(1)
        .all(conn)
        .await;
    let res = match res {
        Ok(model) => model,
        Err(err) => {
            return Err(DBError(err));
        }
    };
    if !res.is_empty() {
        Ok(res[0].to_owned())
    } else {
        Err(Empty)
    }
}

impl Model {
    pub fn check_password(&self, password: String) -> bool {
        let password_stored: Vec<&str> = self.password.split(":").collect();
        if password_stored.len() != 3 {
            false
        } else if password_stored[0] == "sha2" {
            utils::password::check(
                password,
                password_stored[2].to_owned(),
                password_stored[1].to_owned(),
            )
        } else {
            false
        }
    }
}
