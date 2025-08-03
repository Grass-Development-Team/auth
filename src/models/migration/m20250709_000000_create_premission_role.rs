use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Permission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Permission::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Permission::Name)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Permission::Description).string().not_null())
                    .col(ColumnDef::new(Permission::System).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Role::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Role::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Role::Name).string().unique_key().not_null())
                    .col(ColumnDef::new(Role::Description).string().not_null())
                    .col(ColumnDef::new(Role::Level).integer().not_null())
                    .col(ColumnDef::new(Role::System).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RolePermissions::RoleId).uuid().not_null())
                    .col(
                        ColumnDef::new(RolePermissions::PermissionId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk-role-permission")
                            .col(RolePermissions::RoleId)
                            .col(RolePermissions::PermissionId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-role-permission-role_id")
                    .from(RolePermissions::Table, RolePermissions::RoleId)
                    .to(Role::Table, Role::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-role-permission-permission_id")
                    .from(RolePermissions::Table, RolePermissions::PermissionId)
                    .to(Permission::Table, Permission::Id)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-role-permission-role_id")
                    .table(RolePermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-role-permission-permission_id")
                    .table(RolePermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermissions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Permission::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Permission {
    Table,
    Id,
    Name,
    Description,
    System,
}

#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
    Name,
    Description,
    Level,
    System,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}
