use sea_orm::{ActiveValue::Set, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::models::common::ModelError::{self, DBError, Empty, ParamsError};

pub const DEFAULT_LOCALE: &str = "zh-CN";
pub const DEFAULT_TIMEZONE: &str = "Asia/Shanghai";

/// # User Settings Model
#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uid:                i32,
    pub show_email:         bool,
    pub show_gender:        bool,
    pub show_last_login_at: bool,
    pub locale:             String,
    pub timezone:           String,
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
    pub show_last_login_at: bool,
    pub locale:             String,
    pub timezone:           String,
}

impl Default for CreateUserSettingsParams {
    fn default() -> Self {
        Self {
            uid:                0,
            show_email:         false,
            show_gender:        true,
            show_last_login_at: false,
            locale:             DEFAULT_LOCALE.into(),
            timezone:           DEFAULT_TIMEZONE.into(),
        }
    }
}

impl CreateUserSettingsParams {
    fn check(&self) -> bool {
        self.uid > 0 && !self.locale.trim().is_empty() && !self.timezone.trim().is_empty()
    }
}

#[derive(Default)]
pub struct UpdateUserSettingsParams {
    pub show_email:         Option<bool>,
    pub show_gender:        Option<bool>,
    pub show_last_login_at: Option<bool>,
    pub locale:             Option<String>,
    pub timezone:           Option<String>,
}

impl UpdateUserSettingsParams {
    fn check(&self) -> bool {
        if let Some(locale) = &self.locale
            && locale.trim().is_empty()
        {
            return false;
        }

        if let Some(timezone) = &self.timezone
            && timezone.trim().is_empty()
        {
            return false;
        }

        true
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
        uid:                Set(params.uid),
        show_email:         Set(params.show_email),
        show_gender:        Set(params.show_gender),
        show_last_login_at: Set(params.show_last_login_at),
        locale:             Set(params.locale),
        timezone:           Set(params.timezone),
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

pub async fn ensure_user_settings(
    conn: &impl ConnectionTrait,
    uid: i32,
) -> Result<Model, ModelError> {
    match get_user_settings_by_uid(conn, uid).await {
        Ok(settings) => Ok(settings),
        Err(Empty) => create_default_user_settings(conn, uid).await,
        Err(err) => Err(err),
    }
}

pub async fn update_user_settings(
    conn: &impl ConnectionTrait,
    uid: i32,
    params: UpdateUserSettingsParams,
) -> Result<Model, ModelError> {
    if !params.check() {
        return Err(ParamsError);
    }

    let settings = get_user_settings_by_uid(conn, uid).await?;
    let mut settings = settings.into_active_model();

    if let Some(show_email) = params.show_email {
        settings.show_email = Set(show_email);
    }
    if let Some(show_gender) = params.show_gender {
        settings.show_gender = Set(show_gender);
    }
    if let Some(show_last_login_at) = params.show_last_login_at {
        settings.show_last_login_at = Set(show_last_login_at);
    }
    if let Some(locale) = params.locale {
        settings.locale = Set(locale);
    }
    if let Some(timezone) = params.timezone {
        settings.timezone = Set(timezone);
    }

    settings.update(conn).await.map_err(DBError)
}
