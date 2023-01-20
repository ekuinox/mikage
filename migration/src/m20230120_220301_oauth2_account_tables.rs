use sea_orm_migration::prelude::*;

use crate::m20220101_000001_create_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SpotifyAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyAccounts::UserId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::AvatarUrl)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::RefreshToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::OwnerUserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SpotifyAccounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SpotifyAccounts::Table, SpotifyAccounts::OwnerUserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TwitterAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TwitterAccounts::UserId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::ScreenName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::AvatarUrl)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::RefreshToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::OwnerUserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwitterAccounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TwitterAccounts::Table, TwitterAccounts::OwnerUserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SpotifyAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TwitterAccounts::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum SpotifyAccounts {
    Table,
    UserId,
    DisplayName,
    AvatarUrl,
    AccessToken,
    RefreshToken,
    OwnerUserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum TwitterAccounts {
    Table,
    UserId,
    ScreenName,
    DisplayName,
    AvatarUrl,
    AccessToken,
    RefreshToken,
    OwnerUserId,
    CreatedAt,
    UpdatedAt,
}
