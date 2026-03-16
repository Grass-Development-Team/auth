use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserRole::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserRole::UserId).integer().not_null())
                    .col(ColumnDef::new(UserRole::RoleId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-user-role")
                            .col(UserRole::UserId)
                            .col(UserRole::RoleId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-user_role-user_id")
                    .from(UserRole::Table, UserRole::UserId)
                    .to(Users::Table, Users::Uid)
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-user_role-role_id")
                    .from(UserRole::Table, UserRole::RoleId)
                    .to(Role::Table, Role::Id)
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
                    .name("fk-user_role-user_id")
                    .table(UserRole::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-user_role-role_id")
                    .table(UserRole::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserRole::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserRole {
    Table,
    UserId,
    RoleId,
}

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Users {
    Table,
    Uid,
    Email,
    Username,
    Password,
    Nickname,
    Status,
}

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Role {
    Table,
    Id,
    Name,
    Description,
}
