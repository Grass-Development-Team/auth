use crate::models::common::ModelError;
use crate::models::common::ModelError::{DBError, Empty};

use crate::internal::utils;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::{IntoActiveModel, QuerySelect};
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
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::user_info::Entity", on_delete = "Cascade")]
    UserInfo,
    #[sea_orm(has_many = "super::user_role::Entity", on_delete = "Cascade")]
    UserRole,
}

impl Related<super::user_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserInfo.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Get user model by email
pub async fn get_user_by_email(
    conn: &impl ConnectionTrait,
    email: String,
) -> Result<(Model, Vec<super::user_info::Model>), ModelError> {
    let res = Entity::find()
        .find_with_related(super::user_info::Entity)
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

/// Get user model by username
pub async fn get_user_by_username(
    conn: &impl ConnectionTrait,
    username: String,
) -> Result<(Model, Vec<super::user_info::Model>), ModelError> {
    let res = Entity::find()
        .find_with_related(super::user_info::Entity)
        .filter(Column::Username.eq(username))
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

/// Get user model by id
pub async fn get_user_by_id(
    conn: &impl ConnectionTrait,
    id: i32,
) -> Result<(Model, Vec<super::user_info::Model>), ModelError> {
    let res = Entity::find()
        .find_with_related(super::user_info::Entity)
        .filter(Column::Uid.eq(id))
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

pub async fn delete_user(conn: &impl ConnectionTrait, id: i32) -> Result<(), ModelError> {
    let (user, _) = get_user_by_id(conn, id).await?;

    let mut user = user.into_active_model();
    user.status = Set(AccountStatus::Deleted);

    match user.update(conn).await {
        Ok(_) => Ok(()),
        Err(err) => Err(DBError(err)),
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

impl AccountStatus {
    pub fn is_deleted(&self) -> bool {
        matches!(self, AccountStatus::Deleted)
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self, AccountStatus::Inactive)
    }

    pub fn is_banned(&self) -> bool {
        matches!(self, AccountStatus::Banned)
    }
}

impl From<AccountStatus> for &str {
    fn from(status: AccountStatus) -> Self {
        match status {
            AccountStatus::Inactive => "inactive",
            AccountStatus::Active => "active",
            AccountStatus::Banned => "banned",
            AccountStatus::Deleted => "deleted",
        }
    }
}
