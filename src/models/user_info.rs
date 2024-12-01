use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Gender {
    Male = 0,
    Female = 1,
    Custom = 2,
}

/// # User Info Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid: i32,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub gender: Option<Gender>,
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No defined relation")
    }
}

impl ActiveModelBehavior for ActiveModel {}