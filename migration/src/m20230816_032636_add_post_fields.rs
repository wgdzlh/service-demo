use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Posts::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Posts::Table)
                    .drop_column(Posts::CreatedAt)
                    .drop_column(Posts::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    CreatedAt,
    UpdatedAt,
}
