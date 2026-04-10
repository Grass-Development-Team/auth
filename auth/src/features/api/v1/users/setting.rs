use axum::extract::{Path, State};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    internal::error::{AppError, AppErrorKind},
    models::{common::ModelError, user_settings},
    routers::{
        extractor::{LoginAccess, OperatorAccess},
        response::app_error_to_response,
        serializer::{Response, ResponseCode},
    },
    state::AppState,
};

pub async fn controller(login: LoginAccess) -> Response<SettingResponse> {
    let service = SettingService;
    let data = service.setting(login.user.2);

    Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(data))
}

pub async fn controller_by_uid(
    OperatorAccess(_login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
) -> Response<SettingResponse> {
    let service = SettingService;

    match service.setting_by_uid(&state.db, uid).await {
        Ok(data) => Response::new(ResponseCode::OK.into(), ResponseCode::OK.into(), Some(data)),
        Err(err) => app_error_to_response(err),
    }
}

#[derive(Serialize, Deserialize)]
pub struct SettingResponse {
    pub uid:                i32,
    pub show_email:         bool,
    pub show_gender:        bool,
    pub show_state:         bool,
    pub show_last_login_at: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale:             Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone:           Option<String>,
}

impl From<user_settings::Model> for SettingResponse {
    fn from(value: user_settings::Model) -> Self {
        Self {
            uid:                value.uid,
            show_email:         value.show_email,
            show_gender:        value.show_gender,
            show_state:         value.show_state,
            show_last_login_at: value.show_last_login_at,
            locale:             value.locale,
            timezone:           value.timezone,
        }
    }
}

pub struct SettingService;

impl SettingService {
    pub fn setting(&self, settings: user_settings::Model) -> SettingResponse {
        settings.into()
    }

    pub async fn setting_by_uid(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
    ) -> Result<SettingResponse, AppError> {
        match user_settings::get_user_settings_by_uid(conn, uid).await {
            Ok(settings) => Ok(settings.into()),
            Err(ModelError::Empty) => Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "users.setting_by_uid.find_user_setting",
            )),
            Err(err) => Err(AppError::infra(
                AppErrorKind::InternalError,
                "users.setting_by_uid.find_user_setting",
                err,
            )),
        }
    }
}
