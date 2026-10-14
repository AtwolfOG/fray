use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table("client")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("client_secret"))
                    .col(string("host"))
                    .col(string("name"))
                    .to_owned(),
            )
            .await?;
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
                .to_owned()
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table("request").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("client").to_owned())
            .await
    }
}
