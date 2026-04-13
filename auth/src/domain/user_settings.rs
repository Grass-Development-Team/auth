use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::infra::database::ModelError::{self, DBError, Empty, ParamsError};

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

pub struct CreateUserSettingsParams {
    pub uid:                i32,
    pub show_email:         bool,
    pub show_gender:        bool,
    pub show_state:         bool,
    pub show_last_login_at: bool,
    pub locale:             Option<String>,
    pub timezone:           Option<String>,
}

impl Default for CreateUserSettingsParams {
    fn default() -> Self {
        Self {
            uid:                0,
            show_email:         false,
            show_gender:        true,
            show_state:         true,
            show_last_login_at: false,
            locale:             None,
            timezone:           None,
        }
    }
}

impl CreateUserSettingsParams {
    fn check(&self) -> bool {
        self.uid > 0
    }
}

pub async fn get_user_settings_by_uid(
    conn: &impl ConnectionTrait,
    uid: i32,
) -> Result<Model, ModelError> {
    let res = Entity::find_by_id(uid).one(conn).await.map_err(DBError)?;

    if let Some(settings) = res {
        Ok(settings)
    } else {
        Err(Empty)
    }
}

pub async fn create_user_settings(
    conn: &impl ConnectionTrait,
    params: CreateUserSettingsParams,
) -> Result<Model, ModelError> {
    if !params.check() {
        return Err(ParamsError);
    }

    ActiveModel {
        uid: Set(params.uid),
        show_email: Set(params.show_email),
        show_gender: Set(params.show_gender),
        show_state: Set(params.show_state),
        show_last_login_at: Set(params.show_last_login_at),
        locale: Set(params.locale),
        timezone: Set(params.timezone),
        ..Default::default()
    }
    .insert(conn)
    .await
    .map_err(DBError)
}

pub async fn create_default_user_settings(
    conn: &impl ConnectionTrait,
    uid: i32,
) -> Result<Model, ModelError> {
    create_user_settings(
        conn,
        CreateUserSettingsParams {
            uid,
            ..Default::default()
        },
    )
    .await
}
