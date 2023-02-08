pub use sea_orm_migration::prelude::*;
mod migrations;
mod timestamps;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            migrations::m20230203_135154_create_participants::Migration,
        )]
    }
}
