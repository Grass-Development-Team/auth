use crate::internal::config::Database as DatabaseType;
use crate::models::migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, Set};
use sea_orm_migration::MigratorTrait;
use tracing::log;

// Import the permission entity
use crate::models::permission::{ActiveModel as PermissionActiveModel, Entity as Permission};

pub async fn init(sql: &DatabaseType) -> Result<DatabaseConnection, DbErr> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        sql.username, sql.password, sql.host, sql.port, sql.dbname
    );

    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging_level(log::LevelFilter::Debug);

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    init_permissions(&db).await?;

    Ok(db)
}

async fn init_permissions(db: &DatabaseConnection) -> Result<(), DbErr> {
    // List of permissions to initialize
    let permissions = vec![
        // User Management
        ("user:create", "Create new user"),
        ("user:read:self", "Read own user info"),
        ("user:read:all", "Read all users info (admin)"),
        ("user:update:self", "Update own user info"),
        (
            "user:update:all",
            "Update all users info (admin, including disable/activate account)",
        ),
        (
            "user:delete:self",
            "Delete own account (should require confirmation/MFA)",
        ),
        ("user:delete:all", "Delete all users (very high privilege)"),
        ("user:reset_password:self", "Reset own password"),
        (
            "user:reset_password:other",
            "Reset other user's password (admin)",
        ),
        (
            "user:manage_roles",
            "Manage user roles (assign/remove roles, admin only)",
        ),
        (
            "user:manage_mfa:self",
            "Manage own MFA settings (enable/disable/configure TOTP etc)",
        ),
        (
            "user:manage_mfa:other",
            "Manage other user's MFA settings (admin)",
        ),
        // Role Management
        ("role:create", "Create new role"),
        ("role:read", "View role list or single role info"),
        ("role:update", "Update role info"),
        ("role:delete", "Delete role"),
        (
            "role:manage_permissions",
            "Manage role permissions (assign/remove permission points)",
        ),
        // Permission Management
        ("permission:read", "View all permission points (admin only)"),
        (
            "permission:manage",
            "Create/modify/delete permission points (super admin only)",
        ),
    ];

    let existing: Vec<String> = Permission::find()
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.name)
        .collect();

    let existing_set: std::collections::HashSet<String> = existing.into_iter().collect();

    let mut new_permissions = Vec::new();

    for (name, description) in permissions {
        if !existing_set.contains(name) {
            new_permissions.push(PermissionActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
            });
        }
    }

    if !new_permissions.is_empty() {
        Permission::insert_many(new_permissions).exec(db).await?;
    }

    Ok(())
}
