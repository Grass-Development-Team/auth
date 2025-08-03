use sea_orm::{JoinType, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::models::common::ModelError;

/// # Role Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "role")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub level: i32,
    pub system: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role_permissions::Entity")]
    RolePermissions,
    #[sea_orm(has_many = "super::user_role::Entity")]
    UserRole,
}

impl Related<super::role_permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RolePermissions.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Get Role ID by role name.
pub async fn get_role_id(conn: &DatabaseConnection, name: String) -> Result<Uuid, ModelError> {
    let role = Entity::find()
        .filter(Column::Name.eq(name))
        .one(conn)
        .await
        .map_err(ModelError::DBError)?;

    if let Some(role) = role {
        Ok(role.id)
    } else {
        Err(ModelError::Empty)
    }
}

pub async fn get_user_role_level(conn: &DatabaseConnection, uid: i32) -> Result<i32, ModelError> {
    let role = Entity::find()
        .join(JoinType::InnerJoin, Relation::UserRole.def())
        .filter(super::user_role::Column::UserId.eq(uid))
        .all(conn)
        .await
        .map_err(ModelError::DBError)?;

    let level = role.iter().map(|r| r.level).max().unwrap_or(0);

    Ok(level)
}
