use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table("api_logs")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(binary("request_payload"))
                    .col(binary("response_payload"))
                    .col(integer("duration_ms").default(0))
                    .col(enumeration("status", "status", ["success", "failed", "processing"]).default("processing"))
                    .col(timestamp_with_time_zone_default_now("created_at"))
                    .col(timestamp_with_time_zone_default_now("updated_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table("api_logs").to_owned())
            .await
    }
}
