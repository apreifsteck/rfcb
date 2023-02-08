use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use migration::{Migrator, MigratorTrait};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let connection = sea_orm::Database::connect(&database_url)
        .await
        .expect("Could not connect to DB");
    Migrator::up(&connection, None)
        .await
        .expect("Could not run migrations");

    HttpServer::new(|| App::new())
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
