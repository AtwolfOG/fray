use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261014_195758_api_logs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
        .create_table(
            Table::create()
            .table("api_log")
            .if_not_exists()
            .col(pk_auto("id"))
            .col(integer("client_id"))
            .col(binary("request_payload"))
            .col(binary("response_payload"))
            .col(enumeration("status", "status", ["success", "failed", "processing"]).default("processing"))
            .col(timestamp_with_time_zone_default_now("created_at"))
            .col(timestamp_with_time_zone_default_now("updated_at"))
            .foreign_key(
                ForeignKey::create()
                .name("fk-api_log-client")
                .from("api_log", "client_id")
                .to("client", "id")
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
            )
            .take()
        )
        .await?;

        manager
        .create_index(
            Index::create()
            .name("idx-api_log-client_id")
            .if_not_exists()
            .table("api_log")
            .col("client_id")
            .take()
        )
        .await?;

        manager
        .create_index(
            Index::create()
            .name("idx-api_log-created_at")
            .if_not_exists()
            .table("api_log")
            .col("created_at")
            .take()
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("client")
                    .add_column(timestamp_with_time_zone_default_now("created_at"))
                    .take()
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table("client")
                    .add_column(timestamp_with_time_zone_default_now("updated_at"))
                    .take()
            )
            .await?;

        manager.drop_table(Table::drop().table("request").if_exists().take()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager.alter_table(Table::alter().table("client").drop_column("created_at").to_owned()).await?;

        manager.alter_table(Table::alter().table("client").drop_column("updated_at").to_owned()).await?;

        manager
        .create_table(
            Table::create()
                .table("request")
                .if_not_exists()
                .col(pk_auto("id"))
                .col(string("data"))
                .col(integer("client_id"))
                .foreign_key(
                    ForeignKey::create()
                    .name("fk-request-client")
                    .from("request", "client_id")
                    .to("client", "id")
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                )
                .take()
        )
        .await?;

        manager.drop_index(Index::drop().name("idx-api_log-client_id").to_owned()).await?;

        manager.drop_index(Index::drop().name("idx-api_log-created_at").to_owned()).await?;

        manager.drop_table(Table::drop().table("api_log").if_exists().take()).await?;

        Ok(())
    }
}
