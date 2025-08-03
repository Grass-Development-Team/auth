use sea_orm::{JoinType, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::models::common::ModelError;

/// # Permission Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "permission")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub system: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role_permissions::Entity")]
    RolePermissions,
}

impl Related<super::role_permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RolePermissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_permissions_by_uid(
    db: &DatabaseConnection,
    uid: i32,
) -> Result<Vec<String>, ModelError> {
    let permissions = Entity::find()
        .join(JoinType::InnerJoin, Relation::RolePermissions.def())
        .join(
            JoinType::InnerJoin,
            super::role_permissions::Relation::Role.def(),
        )
        .join(JoinType::InnerJoin, super::role::Relation::UserRole.def())
        .filter(super::user_role::Column::UserId.eq(uid))
        .all(db)
        .await
        .map_err(ModelError::DBError)?;

    Ok(permissions
        .into_iter()
        .map(|res| res.name)
        .collect::<Vec<String>>())
}

pub async fn check_permission(db: &DatabaseConnection, uid: i32, perm: &str) -> bool {
    let Ok(permissions) = get_permissions_by_uid(db, uid).await else {
        return false;
    };
    permissions.iter().any(|p| p == perm)
}
