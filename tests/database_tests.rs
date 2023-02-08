use sea_orm::prelude::*;
use std::env;

use migration::{Migrator, MigratorTrait};
async fn main() -> Result<(), DbErr> {
    // Connecting SQLite
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db = sea_orm::Database::connect(&database_url).await?;

    migration::Migrator::up(&db, None).await?;

    // Performing tests

    Ok(())
}
