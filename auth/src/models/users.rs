use crypto::password::PasswordManager;
use sea_orm::{ActiveValue::Set, IntoActiveModel, JoinType, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::models::{
    common::ModelError::{self, DBError, Empty, ParamsError},
    permission, role, user_info, user_role, user_settings,
};

/// Status of the Account
/// - Inactive (0): User haven't active the account through link send to email
/// - Active (1): Active account
/// - Banned (2): Account banned by admin
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum AccountStatus {
    Inactive = 0,
    Active   = 1,
    Banned   = 2,
    Deleted  = 3,
}

/// # Users Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub uid:        i32,
    pub email:      String,
    pub username:   String,
    pub password:   String,
    pub nickname:   String,
    pub status:     AccountStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::user_info::Entity", on_delete = "Cascade")]
    Info,
    #[sea_orm(has_one = "super::user_settings::Entity", on_delete = "Cascade")]
    Settings,
    #[sea_orm(has_many = "super::user_role::Entity", on_delete = "Cascade")]
    Roles,
}

impl Related<super::user_info::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Info.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roles.def()
    }
}

impl Related<super::user_settings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Settings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Get user model by email
pub async fn get_user_by_email(
    conn: &impl ConnectionTrait,
    email: &str,
) -> Result<(Model, super::user_info::Model, super::user_settings::Model), ModelError> {
    let res = Entity::find()
        .find_also_related(super::user_info::Entity)
        .find_also_related(super::user_settings::Entity)
        .filter(Column::Email.eq(email))
        .limit(1)
        .all(conn)
        .await;
    let res = match res {
        Ok(model) => model,
        Err(err) => {
            return Err(DBError(err));
        },
    };

    if res.is_empty() {
        return Err(Empty);
    }

    let res = res[0].to_owned();
    let user = res.0;
    let Some(user_info) = res.1 else {
        tracing::error!("User info not found for user with uid: {}", user.uid);

        return Err(Empty);
    };
    let Some(user_settings) = res.2 else {
        tracing::error!("User settings not found for user with uid: {}", user.uid);

        return Err(Empty);
    };

    Ok((user, user_info, user_settings))
}

/// Get user model by username
pub async fn get_user_by_username(
    conn: &impl ConnectionTrait,
    username: &str,
) -> Result<(Model, super::user_info::Model, super::user_settings::Model), ModelError> {
    let res = Entity::find()
        .find_also_related(super::user_info::Entity)
        .find_also_related(super::user_settings::Entity)
        .filter(Column::Username.eq(username))
        .limit(1)
        .all(conn)
        .await;
    let res = match res {
        Ok(model) => model,
        Err(err) => {
            return Err(DBError(err));
        },
    };

    if res.is_empty() {
        return Err(Empty);
    }

    let res = res[0].to_owned();
    let user = res.0;
    let Some(user_info) = res.1 else {
        tracing::error!("User info not found for user with uid: {}", user.uid);

        return Err(Empty);
    };
    let Some(user_settings) = res.2 else {
        tracing::error!("User settings not found for user with uid: {}", user.uid);

        return Err(Empty);
    };

    Ok((user, user_info, user_settings))
}

/// Get user model by id
pub async fn get_user_by_id(
    conn: &impl ConnectionTrait,
    id: i32,
) -> Result<(Model, super::user_info::Model, super::user_settings::Model), ModelError> {
    let res = Entity::find()
        .find_also_related(super::user_info::Entity)
        .find_also_related(super::user_settings::Entity)
        .filter(Column::Uid.eq(id))
        .limit(1)
        .all(conn)
        .await;
    let res = match res {
        Ok(model) => model,
        Err(err) => {
            return Err(DBError(err));
        },
    };

    if res.is_empty() {
        return Err(Empty);
    }

    let res = res[0].to_owned();
    let user = res.0;
    let Some(user_info) = res.1 else {
        tracing::error!("User info not found for user with uid: {}", user.uid);

        return Err(Empty);
    };
    let Some(user_settings) = res.2 else {
        tracing::error!("User settings not found for user with uid: {}", user.uid);

        return Err(Empty);
    };

    Ok((user, user_info, user_settings))
}

/// Get user model by role
pub async fn get_user_by_role(
    conn: &impl ConnectionTrait,
    role: &str,
) -> Result<Vec<(Model, super::user_info::Model, super::user_settings::Model)>, ModelError> {
    let res = Entity::find()
        .find_also_related(super::user_info::Entity)
        .find_also_related(super::user_settings::Entity)
        .join(JoinType::InnerJoin, Relation::Roles.def())
        .join(JoinType::InnerJoin, super::user_role::Relation::Role.def())
        .filter(super::role::Column::Name.eq(role))
        .all(conn)
        .await
        .map_err(DBError)?;

    Ok(res
        .into_iter()
        .filter_map(|(user, info, settings)| {
            let Some(info) = info else {
                tracing::error!("User info not found for user with uid: {}", user.uid);

                return None;
            };
            let Some(settings) = settings else {
                tracing::error!("User settings not found for user with uid: {}", user.uid);

                return None;
            };

            Some((user, info, settings))
        })
        .collect())
}

pub async fn get_user_status(
    conn: &impl ConnectionTrait,
    id: i32,
) -> Result<AccountStatus, ModelError> {
    let res = get_user_by_id(conn, id).await?;
    Ok(res.0.status)
}

pub struct CreateUserParams {
    pub username: String,
    pub email:    String,
    pub password: String,
    pub status:   AccountStatus,
    pub role:     String,
    pub nickname: Option<String>,
}

impl Default for CreateUserParams {
    fn default() -> Self {
        Self {
            username: Default::default(),
            email:    Default::default(),
            password: Default::default(),
            status:   AccountStatus::Inactive,
            role:     "user".into(),
            nickname: None,
        }
    }
}

impl CreateUserParams {
    fn check(&self) -> bool {
        !self.username.is_empty() && !self.email.is_empty() && !self.password.is_empty()
    }
}

pub async fn create_user(
    conn: &impl ConnectionTrait,
    params: CreateUserParams,
) -> Result<(), ModelError> {
    if !params.check() {
        return Err(ParamsError);
    }

    // Insert User
    let user = ActiveModel {
        username: Set(params.username),
        email: Set(params.email.clone()),
        password: Set(params.password),
        nickname: Set(if let Some(nickname) = params.nickname {
            nickname
        } else {
            params.email.split("@").collect::<Vec<&str>>()[0].to_owned()
        }),
        status: Set(params.status),
        ..Default::default()
    };
    let user = user.insert(conn).await.map_err(ModelError::DBError)?;

    // Insert User Info
    user_info::ActiveModel {
        uid: Set(user.uid),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(ModelError::DBError)?;

    // Insert User Settings
    user_settings::create_default_user_settings(conn, user.uid).await?;

    // Insert User Role
    let role_id = role::get_role_id(conn, params.role).await?;

    user_role::ActiveModel {
        user_id: Set(user.uid),
        role_id: Set(role_id),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(ModelError::DBError)?;

    Ok(())
}

pub async fn delete_user(conn: &impl ConnectionTrait, id: i32) -> Result<(), ModelError> {
    let (user, _, _) = get_user_by_id(conn, id).await?;

    let mut user = user.into_active_model();
    user.status = Set(AccountStatus::Deleted);

    match user.update(conn).await {
        Ok(_) => Ok(()),
        Err(err) => Err(DBError(err)),
    }
}

impl Model {
    pub fn check_password(&self, password: String) -> bool {
        match PasswordManager::verify(&password, &self.password) {
            Ok(res) => res,
            Err(err) => {
                tracing::error!("Password verification failed: {err}");
                false
            },
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

        let salt = PasswordManager::generate_salt();
        let password = PasswordManager::hash(
            crypto::password::PasswordHashAlgorithm::Argon2id,
            &new_password,
            &salt,
        )?;

        user.password = Set(password);

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
