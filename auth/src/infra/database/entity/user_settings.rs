use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// # User Settings Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid:                i32,
    pub show_email:         bool,
    pub show_gender:        bool,
    pub show_state:         bool,
    pub show_last_login_at: bool,
    pub locale:             Option<String>,
    pub timezone:           Option<String>,
    pub created_at:         DateTimeUtc,
    pub updated_at:         DateTimeUtc,
    pub deleted_at:         Option<DateTimeUtc>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::Uid",
        to = "super::users::Column::Uid",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
