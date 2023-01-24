use diesel::connection::LoadConnection;
use diesel::pg::PgConnection;
use diesel::query_builder::{Query, QueryFragment, QueryId};
use diesel::query_dsl::LoadQuery;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, ConnectionResult, QueryResult};
use dotenvy::dotenv;
use std::env;

pub struct TestDB {
    url: String,
}

impl DB<PgConnection> for TestDB {
    fn init() -> Self {
        Self { url: get_db_url() }
    }
    fn get_connection(&self) -> ConnectionResult<PgConnection> {
        PgConnection::establish(&self.url)
    }
}

fn get_db_url() -> String {
    dotenv().ok();

    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
fn create_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = get_db_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

pub trait DB<ConnectionType>
where
    ConnectionType: diesel::Connection + LoadConnection,
{
    fn init() -> Self;
    fn get_connection(&self) -> ConnectionResult<ConnectionType>;

    fn execute<Query>(&self, query: Query) -> QueryResult<usize>
    where
        Query: diesel::prelude::RunQueryDsl<ConnectionType>
            + QueryFragment<<ConnectionType as diesel::Connection>::Backend>
            + QueryId,
    {
        let mut conn = self
            .get_connection()
            .expect("Error connecting to the database");
        query.execute(&mut conn)
    }
    fn get_result<Query>(&self, query: Query) -> QueryResult<usize>
    where
        Query:
            diesel::prelude::RunQueryDsl<ConnectionType> + diesel::query_builder::Query + LoadQuery,
    {
        let mut conn = self
            .get_connection()
            .expect("Error connecting to the database");
        query.get_result(&mut conn)
    }
}

// PLAN:
// - Have a module that wraps the database
// - init the database by creating a connection pool and a
// multiple producer single consumer channel
// - have functions that block the current thread, and send the query
// - to the database processor
// - Have the db process check out a connection and run the query, then return
// the results
