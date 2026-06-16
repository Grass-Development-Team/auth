use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
