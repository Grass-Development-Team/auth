use sea_orm::{JoinType, QuerySelect, entity::prelude::*};

use crate::infra::database::{
    ModelError,
    entity::{permission, role, role_permissions, user_role},
};

pub async fn get_permissions_by_uid(
    db: &impl ConnectionTrait,
    uid: i32,
) -> Result<Vec<String>, ModelError> {
    let permissions = permission::Entity::find()
        .join(
            JoinType::InnerJoin,
            permission::Relation::RolePermissions.def(),
        )
        .join(JoinType::InnerJoin, role_permissions::Relation::Role.def())
        .join(JoinType::InnerJoin, role::Relation::UserRole.def())
        .filter(user_role::Column::UserId.eq(uid))
        .all(db)
        .await
        .map_err(ModelError::DBError)?;

    Ok(permissions
        .into_iter()
        .map(|res| res.name)
        .collect::<Vec<String>>())
}

pub async fn check_permission(db: &impl ConnectionTrait, uid: i32, perm: &str) -> bool {
    let Ok(permissions) = get_permissions_by_uid(db, uid).await else {
        return false;
    };
    permissions.iter().any(|p| p == perm)
}
