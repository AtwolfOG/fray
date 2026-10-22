pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20261014_195758_api_logs;
mod m20261017_213358_add_time_monitoring_to_api_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20261014_195758_api_logs::Migration),
            Box::new(m20261017_213358_add_time_monitoring_to_api_logs::Migration),
        ]
    }
}
