use sea_query::{Alias, Expr, PostgresQueryBuilder, Query};
use sqlx::Executor;
use sqlx::{postgres::PgRow, PgPool};

#[derive(Debug)]
pub enum ChangeError {
    DBError(sqlx::Error),
    ValidationError(Vec<&'static str>),
}
pub trait Insertable {
    fn to_sql(&self) -> String;
}
pub trait Queryable {
    fn to_sql(&self) -> String;
}
pub trait TableRef {
    fn table_ref() -> Alias;
}
impl Queryable for sea_query::SelectStatement {
    fn to_sql(&self) -> String {
        self.to_string(PostgresQueryBuilder)
    }
}
pub trait Validatable {
    fn validate(&self) -> Result<(), ChangeError>;
}
pub async fn insert<R, D>(pool: &PgPool, data: D) -> Result<R, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow>,
    D: Insertable + Validatable,
{
    data.validate()?;
    match pool.fetch_one(data.to_sql().as_str()).await {
        Ok(row) => try_from_row(&row),
        Err(err) => Err(db_error(err)),
    }
}

pub async fn one<R>(pool: &PgPool, query: impl Queryable) -> Result<R, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow>,
{
    match pool.fetch_one(query.to_sql().as_str()).await {
        Ok(row) => try_from_row(&row),
        Err(err) => Err(db_error(err)),
    }
}

pub async fn find<R>(pool: &PgPool, id: i32) -> Result<R, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow> + TableRef,
{
    let query = Query::select()
        .from(R::table_ref())
        .expr(Expr::asterisk())
        .and_where(Expr::col(Alias::new("id")).eq(id))
        .to_owned();
    one::<R>(&pool, query).await
}

fn try_from_row<R>(row: &PgRow) -> Result<R, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow>,
{
    R::from_row(row).map_err(|err| db_error(err))
}

fn db_error(e: sqlx::Error) -> ChangeError {
    ChangeError::DBError(e)
}
