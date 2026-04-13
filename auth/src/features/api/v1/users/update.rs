use axum::extract::{Path, State};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, IntoActiveModel,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use validator::Validatable;

use crate::{
    domain::{
        role,
        user_info::{self, Gender},
        user_settings, users,
    },
    infra::{
        error::{AppError, AppErrorKind},
        http::{
            extractor::{Json, LoginAccess, OperatorAccess},
            response::app_error_to_response,
            serializer::{Response, ResponseCode},
        },
    },
    state::AppState,
};

pub async fn controller(
    login: LoginAccess,
    State(state): State<AppState>,
    Json(req): Json<UpdateService>,
) -> Response {
    match req
        .update(&state.db, login.user.0, login.user.1, login.user.2)
        .await
    {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}

pub async fn controller_by_uid(
    OperatorAccess(login): OperatorAccess,
    State(state): State<AppState>,
    Path(uid): Path<i32>,
    Json(req): Json<UpdateService>,
) -> Response {
    match req.update_by_uid(&state.db, uid, login.level).await {
        Ok(()) => ResponseCode::OK.into(),
        Err(err) => app_error_to_response(err),
    }
}

#[derive(Deserialize, Serialize)]
pub struct UpdateService {
    pub nickname:           Option<String>,
    pub avatar:             Option<String>,
    pub description:        Option<String>,
    pub state:              Option<String>,
    pub gender:             Option<Gender>,
    pub show_email:         Option<bool>,
    pub show_gender:        Option<bool>,
    pub show_state:         Option<bool>,
    pub show_last_login_at: Option<bool>,
    pub locale:             Option<String>,
    pub timezone:           Option<String>,
}

impl UpdateService {
    fn normalize_optional_setting(value: &str) -> Option<String> {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_owned())
        }
    }

    pub async fn update(
        &self,
        conn: &DatabaseConnection,
        user: users::Model,
        info: user_info::Model,
        settings: user_settings::Model,
    ) -> Result<(), AppError> {
        if user.status.is_inactive() {
            return Err(AppError::biz(
                AppErrorKind::UserNotActivated,
                "users.update.check_user_status",
            ));
        }

        if user.status.is_banned() {
            return Err(AppError::biz(
                AppErrorKind::UserBlocked,
                "users.update.check_user_status",
            ));
        }

        self.validate()?;

        let mut user = user.into_active_model();

        if let Some(nickname) = &self.nickname {
            user.nickname = Set(nickname.clone());
        }

        let mut info = info.into_active_model();

        if let Some(avatar) = &self.avatar {
            info.avatar = Set(if avatar.is_empty() {
                None
            } else {
                Some(avatar.clone())
            });
        }

        if let Some(description) = &self.description {
            info.description = Set(if description.is_empty() {
                None
            } else {
                Some(description.clone())
            });
        }

        if let Some(state) = &self.state {
            info.state = Set(if state.is_empty() {
                None
            } else {
                Some(state.clone())
            });
        }

        if let Some(gender) = &self.gender {
            info.gender = Set(Some(gender.clone()));
        }

        let mut settings = settings.into_active_model();

        if let Some(show_email) = self.show_email {
            settings.show_email = Set(show_email);
        }

        if let Some(show_gender) = self.show_gender {
            settings.show_gender = Set(show_gender);
        }

        if let Some(show_state) = self.show_state {
            settings.show_state = Set(show_state);
        }

        if let Some(show_last_login_at) = self.show_last_login_at {
            settings.show_last_login_at = Set(show_last_login_at);
        }

        if let Some(locale) = &self.locale {
            settings.locale = Set(Self::normalize_optional_setting(locale));
        }

        if let Some(timezone) = &self.timezone {
            settings.timezone = Set(Self::normalize_optional_setting(timezone));
        }

        let res = conn
            .transaction(|txn| {
                Box::pin(async move {
                    user.update(txn).await?;
                    info.update(txn).await?;
                    settings.update(txn).await?;
                    Ok::<(), DbErr>(())
                })
            })
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(AppError::infra(
                AppErrorKind::InternalError,
                "users.update.persist",
                err,
            )),
        }
    }

    pub async fn update_by_uid(
        &self,
        conn: &DatabaseConnection,
        uid: i32,
        op_level: i32,
    ) -> Result<(), AppError> {
        let Ok(user) = users::get_user_by_id(conn, uid).await else {
            return Err(AppError::biz(
                AppErrorKind::UserNotFound,
                "users.update_by_uid.find_user",
            ));
        };

        let level = role::get_user_role_level(conn, uid).await.map_err(|err| {
            AppError::infra(
                AppErrorKind::InternalError,
                "users.update_by_uid.role_level",
                err,
            )
        })?;

        if op_level < level {
            return Err(AppError::biz(
                AppErrorKind::Forbidden,
                "users.update_by_uid.check_permission",
            ));
        }

        self.update(conn, user.0, user.1, user.2).await
    }
}

impl Validatable<AppError> for UpdateService {
    fn validate(&self) -> Result<(), AppError> {
        if let Some(nickname) = &self.nickname
            && nickname.len() < 3
        {
            return Err(
                AppError::biz(AppErrorKind::ParamError, "users.update.validate")
                    .with_detail("Nickname should be at least 3 characters long"),
            );
        }

        Ok(())
    }
}
