use colored::Colorize;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;
use tracing::{info, log};
use uuid::Uuid;

use crate::internal::config::Database as DatabaseType;
use crate::internal::utils;

use crate::models::migration::Migrator;
use crate::models::users::{self, AccountStatus};

// Import the permission entity
use crate::models::permission::{ActiveModel as PermissionActiveModel, Entity as Permission};
// Import the role entity
use crate::models::role::{ActiveModel as RoleActiveModel, Entity as Role};
// Import the role_permissions entity
use crate::models::role_permissions::{
    ActiveModel as RolePermissionActiveModel, Entity as RolePermission,
};
// Import user related entities
use crate::models::user_info::ActiveModel as UserInfoActiveModel;
use crate::models::user_role::ActiveModel as UserRoleActiveModel;
use crate::models::users::ActiveModel as UserActiveModel;

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
    init_roles(&db).await?;
    init_role_permissions(&db).await?;
    init_super_admin(&db).await?;

    Ok(db)
}

async fn init_permissions(db: &DatabaseConnection) -> Result<(), DbErr> {
    // List of permissions to initialize
    let permissions = vec![
        // User Management
        ("user:create", "Create new user"),
        ("user:read:self", "Read own user info"),
        ("user:read:active", "Read all active users info"),
        ("user:read:all", "Read all users info"),
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
        ("user:undeletable", "Undeletable user"),
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
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                system: Set(true),
            });
        }
    }

    if !new_permissions.is_empty() {
        Permission::insert_many(new_permissions).exec(db).await?;
    }

    Ok(())
}

async fn init_roles(db: &DatabaseConnection) -> Result<(), DbErr> {
    // List of roles to initialize
    let roles = vec![
        (
            "super_admin",
            "Super Administrator - Full system access",
            100,
        ),
        ("admin", "Administrator - Manage users and roles", 90),
        ("user", "Regular User - Basic user privileges", 10),
    ];

    let existing: Vec<String> = Role::find()
        .all(db)
        .await?
        .into_iter()
        .map(|r| r.name)
        .collect();

    let existing_set: std::collections::HashSet<String> = existing.into_iter().collect();

    let mut new_roles = Vec::new();

    for (name, description, level) in roles {
        if !existing_set.contains(name) {
            new_roles.push(RoleActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                description: Set(description.to_string()),
                level: Set(level),
                system: Set(true),
            });
        }
    }

    if !new_roles.is_empty() {
        Role::insert_many(new_roles).exec(db).await?;
    }

    Ok(())
}

async fn init_super_admin(db: &DatabaseConnection) -> Result<(), DbErr> {
    // TODO: Check super admin by role.

    // Check if super admin already exists
    let super_admin_username = "root";
    let super_admin_email = "admin@local.email";

    // Check if user already exists
    if users::get_user_by_username(db, super_admin_username.to_string())
        .await
        .is_ok()
    {
        return Ok(());
    }

    if users::get_user_by_email(db, super_admin_email.to_string())
        .await
        .is_ok()
    {
        return Ok(());
    }

    // Generate password
    let default_password = utils::rand::string(16);
    let salt = utils::rand::string(16);
    let password = utils::password::generate(default_password.to_owned(), salt.to_owned());

    // Create super admin user
    let super_admin_user = UserActiveModel {
        username: Set(super_admin_username.to_string()),
        email: Set(super_admin_email.to_string()),
        password: Set(format!("sha2:{password}:{salt}")),
        nickname: Set("Super Administrator".to_string()),
        status: Set(AccountStatus::Active),
        ..Default::default()
    };

    let user = super_admin_user.insert(db).await?;

    // Create user info
    let user_info = UserInfoActiveModel {
        uid: Set(user.uid),
        description: Set(Some("System Super Administrator".to_string())),
        ..Default::default()
    };

    user_info.insert(db).await?;

    // Get super_admin role ID
    let super_admin_role = Role::find()
        .filter(crate::models::role::Column::Name.eq("super_admin"))
        .one(db)
        .await?;

    if let Some(role) = super_admin_role {
        // Assign super_admin role to the user
        let user_role = UserRoleActiveModel {
            user_id: Set(user.uid),
            role_id: Set(role.id),
        };

        user_role.insert(db).await?;
    }

    info!("Super admin account created successfully");
    info!("Username: {}", super_admin_username.green());
    info!("Email: {}", super_admin_email.green());
    info!("Default Password: {}", default_password.green());

    Ok(())
}

async fn init_role_permissions(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Get all roles and permissions
    let roles = Role::find().all(db).await?;
    let permissions = Permission::find().all(db).await?;

    // Create maps for easier lookup
    let role_map: std::collections::HashMap<String, Uuid> =
        roles.into_iter().map(|r| (r.name, r.id)).collect();

    let permission_map: std::collections::HashMap<String, Uuid> =
        permissions.into_iter().map(|p| (p.name, p.id)).collect();

    // Define role-permission mappings
    let role_permissions = vec![
        // super_admin gets all permissions
        (
            "super_admin",
            vec![
                "user:create",
                "user:read:self",
                "user:read:active",
                "user:read:all",
                "user:update:self",
                "user:update:all",
                "user:delete:self",
                "user:delete:all",
                "user:undeletable",
                "user:reset_password:self",
                "user:reset_password:other",
                "user:manage_roles",
                "user:manage_mfa:self",
                "user:manage_mfa:other",
                "role:create",
                "role:read",
                "role:update",
                "role:delete",
                "role:manage_permissions",
                "permission:read",
                "permission:manage",
            ],
        ),
        // admin gets most permissions except super admin ones
        (
            "admin",
            vec![
                "user:create",
                "user:read:self",
                "user:read:active",
                "user:read:all",
                "user:update:self",
                "user:update:all",
                "user:delete:self",
                "user:reset_password:self",
                "user:reset_password:other",
                "user:manage_roles",
                "user:manage_mfa:self",
                "user:manage_mfa:other",
                "role:create",
                "role:read",
                "role:update",
                "role:delete",
                "role:manage_permissions",
                "permission:read",
            ],
        ),
        // user gets basic permissions
        (
            "user",
            vec![
                "user:read:self",
                "user:read:active",
                "user:update:self",
                "user:delete:self",
                "user:reset_password:self",
                "user:manage_mfa:self",
            ],
        ),
    ];

    // Get existing role-permission relationships
    let existing_relationships: Vec<(Uuid, Uuid)> = RolePermission::find()
        .all(db)
        .await?
        .into_iter()
        .map(|rp| (rp.role_id, rp.permission_id))
        .collect();

    let existing_set: std::collections::HashSet<(Uuid, Uuid)> =
        existing_relationships.into_iter().collect();

    let mut new_role_permissions = Vec::new();

    for (role_name, permission_names) in role_permissions {
        if let Some(&role_id) = role_map.get(role_name) {
            for permission_name in permission_names {
                if let Some(&permission_id) = permission_map.get(permission_name) {
                    let relationship = (role_id, permission_id);
                    if !existing_set.contains(&relationship) {
                        new_role_permissions.push(RolePermissionActiveModel {
                            role_id: Set(role_id),
                            permission_id: Set(permission_id),
                        });
                    }
                }
            }
        }
    }

    if !new_role_permissions.is_empty() {
        RolePermission::insert_many(new_role_permissions)
            .exec(db)
            .await?;
    }

    Ok(())
}
