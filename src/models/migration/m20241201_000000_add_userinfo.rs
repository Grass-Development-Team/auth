use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserInfo::Uid)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserInfo::Avatar).string())
                    .col(ColumnDef::new(UserInfo::Description).string())
                    .col(ColumnDef::new(UserInfo::State).string())
                    .col(ColumnDef::new(UserInfo::Gender).integer())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-user_info-uid")
                    .from(UserInfo::Table, UserInfo::Uid)
                    .to(Users::Table, Users::Uid)
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
                    .name("fk-user_info-uid")
                    .table(UserInfo::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserInfo::Table).to_owned())
            .await?;

        Ok(())
    }
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

#[derive(DeriveIden)]
enum UserInfo {
    Table,
    Uid,
    Avatar,
    Description,
    State,
    Gender,
}
