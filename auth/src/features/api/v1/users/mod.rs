use axum::{
    Router,
    routing::{any, delete, patch},
};

use crate::{routers::middleware::permission::PermissionAccess, state::AppState};

mod delete;
mod info;
mod setting;
mod update;

pub fn router() -> Router<AppState> {
    let route = Router::new()
        .route(
            "/info",
            any(info::controller).layer(PermissionAccess::all(&["user:read:self"])),
        )
        .route(
            "/info/{uid}",
            any(info::controller_by_uid).layer(PermissionAccess::any(&[
                "user:read:active",
                "user:read:all",
            ])),
        )
        .route(
            "/setting",
            any(setting::controller).layer(PermissionAccess::all(&["user:read:self"])),
        )
        .route(
            "/setting/{uid}",
            any(setting::controller_by_uid).layer(PermissionAccess::all(&["user:read:all"])),
        )
        .route(
            "/delete",
            delete(delete::controller).layer(PermissionAccess::all(&["user:delete:self"])),
        )
        .route(
            "/delete/{uid}",
            delete(delete::controller_by_uid).layer(PermissionAccess::all(&["user:delete:all"])),
        )
        .route(
            "/update",
            patch(update::controller).layer(PermissionAccess::all(&["user:update:self"])),
        )
        .route(
            "/update/{uid}",
            patch(update::controller_by_uid).layer(PermissionAccess::all(&["user:update:all"])),
        );
    Router::new().nest("/user", route)
}
