use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSettings::Uid)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSettings::ShowEmail)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserSettings::ShowGender)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserSettings::ShowState)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserSettings::ShowLastLoginAt)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Locale)
                            .string()
                            .not_null()
                            .default("zh-CN"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Timezone)
                            .string()
                            .not_null()
                            .default("Asia/Shanghai"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-user_settings-uid")
                    .from(UserSettings::Table, UserSettings::Uid)
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
                    .name("fk-user_settings-uid")
                    .table(UserSettings::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(UserSettings::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserSettings {
    Table,
    Uid,
    ShowEmail,
    ShowGender,
    ShowState,
    ShowLastLoginAt,
    Locale,
    Timezone,
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
