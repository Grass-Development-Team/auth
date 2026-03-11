use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

const TABLES: &[&str] = &[
    "users",
    "user_info",
    "user_settings",
    "permission",
    "role",
    "role_permissions",
    "user_role",
];

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        add_audit_columns(
            manager,
            Users::Table,
            Users::CreatedAt,
            Users::UpdatedAt,
            Users::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            UserInfo::Table,
            UserInfo::CreatedAt,
            UserInfo::UpdatedAt,
            UserInfo::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            UserSettings::Table,
            UserSettings::CreatedAt,
            UserSettings::UpdatedAt,
            UserSettings::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            Permission::Table,
            Permission::CreatedAt,
            Permission::UpdatedAt,
            Permission::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            Role::Table,
            Role::CreatedAt,
            Role::UpdatedAt,
            Role::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            RolePermissions::Table,
            RolePermissions::CreatedAt,
            RolePermissions::UpdatedAt,
            RolePermissions::DeletedAt,
        )
        .await?;
        add_audit_columns(
            manager,
            UserRole::Table,
            UserRole::CreatedAt,
            UserRole::UpdatedAt,
            UserRole::DeletedAt,
        )
        .await?;
        create_audit_triggers(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        drop_audit_triggers(manager).await?;
        drop_audit_columns(
            manager,
            UserRole::Table,
            UserRole::CreatedAt,
            UserRole::UpdatedAt,
            UserRole::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            RolePermissions::Table,
            RolePermissions::CreatedAt,
            RolePermissions::UpdatedAt,
            RolePermissions::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            Role::Table,
            Role::CreatedAt,
            Role::UpdatedAt,
            Role::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            Permission::Table,
            Permission::CreatedAt,
            Permission::UpdatedAt,
            Permission::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            UserSettings::Table,
            UserSettings::CreatedAt,
            UserSettings::UpdatedAt,
            UserSettings::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            UserInfo::Table,
            UserInfo::CreatedAt,
            UserInfo::UpdatedAt,
            UserInfo::DeletedAt,
        )
        .await?;
        drop_audit_columns(
            manager,
            Users::Table,
            Users::CreatedAt,
            Users::UpdatedAt,
            Users::DeletedAt,
        )
        .await?;

        Ok(())
    }
}

async fn create_audit_triggers(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION gdt_set_audit_timestamps()
            RETURNS TRIGGER AS $$
            DECLARE
                new_json jsonb;
                old_json jsonb;
                new_status integer;
                old_status integer;
            BEGIN
                IF TG_OP = 'INSERT' THEN
                    NEW.created_at := COALESCE(NEW.created_at, CURRENT_TIMESTAMP);
                    NEW.updated_at := COALESCE(NEW.updated_at, NEW.created_at, CURRENT_TIMESTAMP);

                    IF NEW.deleted_at IS NOT NULL THEN
                        NEW.deleted_at := CURRENT_TIMESTAMP;
                    END IF;

                    RETURN NEW;
                END IF;

                NEW.created_at := OLD.created_at;
                NEW.updated_at := CURRENT_TIMESTAMP;

                IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
                    NEW.deleted_at := CURRENT_TIMESTAMP;
                END IF;

                new_json := to_jsonb(NEW);
                old_json := to_jsonb(OLD);

                IF NEW.deleted_at IS NULL
                    AND new_json ? 'status'
                    AND old_json ? 'status'
                THEN
                    new_status := NULLIF(new_json->>'status', '')::integer;
                    old_status := NULLIF(old_json->>'status', '')::integer;

                    IF new_status = 3 AND COALESCE(old_status, -1) <> 3 THEN
                        NEW.deleted_at := CURRENT_TIMESTAMP;
                    END IF;
                END IF;

                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

    manager
        .get_connection()
        .execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION gdt_soft_delete_row()
            RETURNS TRIGGER AS $$
            BEGIN
                EXECUTE format(
                    'UPDATE %I.%I SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE ctid = $1 AND deleted_at IS NULL',
                    TG_TABLE_SCHEMA,
                    TG_TABLE_NAME
                ) USING OLD.ctid;

                RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

    for table in TABLES {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"DROP TRIGGER IF EXISTS "trg_{table}_audit_timestamps" ON "{table}";"#,
            ))
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                CREATE TRIGGER "trg_{table}_audit_timestamps"
                BEFORE INSERT OR UPDATE ON "{table}"
                FOR EACH ROW
                EXECUTE FUNCTION gdt_set_audit_timestamps();
                "#,
            ))
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"DROP TRIGGER IF EXISTS "trg_{table}_soft_delete" ON "{table}";"#,
            ))
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                CREATE TRIGGER "trg_{table}_soft_delete"
                BEFORE DELETE ON "{table}"
                FOR EACH ROW
                EXECUTE FUNCTION gdt_soft_delete_row();
                "#,
            ))
            .await?;
    }

    Ok(())
}

async fn drop_audit_triggers(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in TABLES {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"DROP TRIGGER IF EXISTS "trg_{table}_soft_delete" ON "{table}";"#,
            ))
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"DROP TRIGGER IF EXISTS "trg_{table}_audit_timestamps" ON "{table}";"#,
            ))
            .await?;
    }

    manager
        .get_connection()
        .execute_unprepared("DROP FUNCTION IF EXISTS gdt_soft_delete_row();")
        .await?;
    manager
        .get_connection()
        .execute_unprepared("DROP FUNCTION IF EXISTS gdt_set_audit_timestamps();")
        .await?;

    Ok(())
}

async fn add_audit_columns<T>(
    manager: &SchemaManager<'_>,
    table: T,
    created_at: T,
    updated_at: T,
    deleted_at: T,
) -> Result<(), DbErr>
where
    T: Iden + Copy + 'static,
{
    let table_name = table.to_string();
    let created_at_name = created_at.to_string();
    let updated_at_name = updated_at.to_string();
    let deleted_at_name = deleted_at.to_string();

    if !manager.has_column(&table_name, &created_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .add_column(
                        ColumnDef::new(created_at)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
    }

    if !manager.has_column(&table_name, &updated_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .add_column(
                        ColumnDef::new(updated_at)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
    }

    if !manager.has_column(&table_name, &deleted_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .add_column(ColumnDef::new(deleted_at).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;
    }

    Ok(())
}

async fn drop_audit_columns<T>(
    manager: &SchemaManager<'_>,
    table: T,
    created_at: T,
    updated_at: T,
    deleted_at: T,
) -> Result<(), DbErr>
where
    T: Iden + Copy + 'static,
{
    let table_name = table.to_string();
    let created_at_name = created_at.to_string();
    let updated_at_name = updated_at.to_string();
    let deleted_at_name = deleted_at.to_string();

    if manager.has_column(&table_name, &deleted_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .drop_column(deleted_at)
                    .to_owned(),
            )
            .await?;
    }

    if manager.has_column(&table_name, &updated_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .drop_column(updated_at)
                    .to_owned(),
            )
            .await?;
    }

    if manager.has_column(&table_name, &created_at_name).await? {
        manager
            .alter_table(
                Table::alter()
                    .table(table)
                    .drop_column(created_at)
                    .to_owned(),
            )
            .await?;
    }

    Ok(())
}

#[derive(DeriveIden, Clone, Copy)]
enum Users {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum UserInfo {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum UserSettings {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum Permission {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum Role {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum RolePermissions {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum UserRole {
    Table,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
