use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Status of the Account
/// - Inactive (0): User haven't active the account through link send to email
/// - Active (1): Active account
/// - Banned (2): Account banned by admin
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
pub enum AccountStatus {
    Inactive = 0,
    Active = 1,
    Banned = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
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
    #[sea_orm(primary_key)]
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