use crate::error::ChangeError;
use sea_query::{Alias, InsertStatement, PostgresQueryBuilder, Query};
use sqlx::Database;
use std::fmt::Debug;
pub trait Entity<D: Database>: Sized {
    type Record: DBRecord<D> + Into<Self>;
}

pub trait DBRecord<D: Database>: for<'b> sqlx::FromRow<'b, D::Row> {
    fn table_ref() -> Alias;
    fn primary_key(&self) -> i32;
}

pub trait Insertable<D: Database>: Sized {
    type Output: DBRecord<D>;
    fn inject_values<'a>(self, starter_query: &'a mut InsertStatement) -> &mut InsertStatement;
    fn prepare_query(self) -> String {
        let mut starting_query = Query::insert();
        starting_query.into_table(Self::Output::table_ref());
        self.inject_values(&mut starting_query);
        starting_query
            .returning(Query::returning().all())
            .to_string(PostgresQueryBuilder)
    }
}
pub trait Queryable<D: Database>: Debug {
    type Output: DBRecord<D>;
    fn to_sql(&self) -> String;
}

pub trait Validatable {
    fn validate(&self) -> Result<(), ChangeError>;
}
