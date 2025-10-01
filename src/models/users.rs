use crate::models::common::ModelError;
use crate::models::common::ModelError::{DBError, Empty};

use crate::internal::utils;
use crate::models::{permission, role, user_info, user_role};
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

pub async fn get_user_status(
    conn: &impl ConnectionTrait,
    id: i32,
) -> Result<AccountStatus, ModelError> {
    let res = get_user_by_id(conn, id).await?;
    Ok(res.0.status)
}

pub async fn create_user(
    conn: &impl ConnectionTrait,
    username: String,
    email: String,
    password: String,
    salt: String,
    status: AccountStatus,
    nickname: Option<String>,
) -> Result<(), ModelError> {
    // Insert User
    let user = ActiveModel {
        username: Set(username),
        email: Set(email.clone()),
        password: Set(format!("sha2:{password}:{salt}")),
        nickname: Set(if let Some(nickname) = nickname {
            nickname
        } else {
            email.split("@").collect::<Vec<&str>>()[0].to_owned()
        }),
        status: Set(status),
        ..Default::default()
    };
    let user = user.insert(conn).await.map_err(ModelError::DBError)?;

    // Insert User Info
    let info = user_info::ActiveModel {
        uid: Set(user.uid),
        ..Default::default()
    };
    info.insert(conn).await.map_err(ModelError::DBError)?;

    // Insert User Role
    // TODO: Default Role setting
    let role_id = role::get_role_id(conn, "user".into()).await?;

    let role = user_role::ActiveModel {
        user_id: Set(user.uid),
        role_id: Set(role_id),
    };
    role.insert(conn).await.map_err(ModelError::DBError)?;

    Ok(())
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

    pub async fn check_permission(&self, conn: &impl ConnectionTrait, perm: &str) -> bool {
        permission::check_permission(conn, self.uid, perm).await
    }

    pub async fn update_password(
        &self,
        conn: &impl ConnectionTrait,
        new_password: String,
    ) -> Result<Model, ModelError> {
        let mut user = self.clone().into_active_model();

        let salt = utils::rand::string(16);
        let password = utils::password::generate(new_password.to_owned(), salt.to_owned());

        user.password = Set(format!("sha2:{password}:{salt}"));

        user.update(conn).await.map_err(ModelError::DBError)
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
