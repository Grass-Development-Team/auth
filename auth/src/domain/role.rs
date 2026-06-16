use sea_orm::{JoinType, QuerySelect, entity::prelude::*};

use crate::infra::database::{
    ModelError,
    entity::{role, user_role},
};

/// Get Role ID by role name.
pub async fn get_role_id(conn: &impl ConnectionTrait, name: String) -> Result<Uuid, ModelError> {
    let role = role::Entity::find()
        .filter(role::Column::Name.eq(name))
        .one(conn)
        .await
        .map_err(ModelError::DBError)?;

    if let Some(role) = role {
        Ok(role.id)
    } else {
        Err(ModelError::Empty)
    }
}

pub async fn get_user_role_level(conn: &impl ConnectionTrait, uid: i32) -> Result<i32, ModelError> {
    let role = role::Entity::find()
        .join(JoinType::InnerJoin, role::Relation::UserRole.def())
        .filter(user_role::Column::UserId.eq(uid))
        .all(conn)
        .await
        .map_err(ModelError::DBError)?;

    let level = role.iter().map(|r| r.level).max().unwrap_or(0);

    Ok(level)
}
