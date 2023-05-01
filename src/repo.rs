use sea_query::{Alias, InsertStatement, PostgresQueryBuilder, Query};
use sqlx::Executor;
use sqlx::{postgres::PgRow, PgPool};

#[derive(Debug)]
pub enum ChangeError {
    DBError(sqlx::Error),
    // TODO will probably have to change this once we actually put validation on
    // some of these things
    ValidationError(Vec<&'static str>),
}

impl From<sqlx::Error> for ChangeError {
    fn from(value: sqlx::Error) -> Self {
        ChangeError::DBError(value)
    }
}

#[derive(Debug, Default)]
pub enum Loadable<T> {
    #[default]
    NotLoaded,
    Loaded(Option<T>),
}

pub trait DBRecord: for<'b> sqlx::FromRow<'b, PgRow> {
    fn table_ref() -> Alias;
    fn primary_key(&self) -> i32;
}

pub trait Insertable: Sized {
    type Output: DBRecord;
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
pub trait Queryable {
    type Output: DBRecord;
    fn to_sql(&self) -> String;
}
// impl Queryable for sea_query::SelectStatement {
//     fn to_sql(&self) -> String {
//         self.to_string(PostgresQueryBuilder)
//     }
// }
pub trait Validatable {
    fn validate(&self) -> Result<(), ChangeError>;
}
pub async fn insert<I, D>(pool: &PgPool, data: D) -> Result<D::Output, ChangeError>
where
    I: for<'b> sqlx::FromRow<'b, PgRow>,
    D: Insertable<Output = I> + Validatable,
{
    data.validate()?;
    let row = pool.fetch_one(data.prepare_query().as_str()).await?;
    let obj: D::Output = <D as Insertable>::Output::from_row(&row)?;
    Ok(obj)
}

pub async fn all<R>(pool: &PgPool, query: impl Queryable) -> Result<Vec<R>, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow>,
{
    let rows = pool.fetch_all(query.to_sql().as_str()).await?;
    //TODO: figure out what to do if a read fails
    Ok(rows
        .into_iter()
        .map(|row| R::from_row(&row).unwrap())
        .collect())
}

pub async fn one<R>(pool: &PgPool, query: impl Queryable) -> Result<R, ChangeError>
where
    R: for<'b> sqlx::FromRow<'b, PgRow>,
{
    let row = pool.fetch_one(query.to_sql().as_str()).await?;
    let obj: R = R::from_row(&row)?;
    Ok(obj)
}

// pub async fn find<R>(pool: &PgPool, id: i32) -> Result<R, ChangeError>
// where
//     R: for<'b> sqlx::FromRow<'b, PgRow> + TableRef,
// {
//     let query = Query::select()
//         .from(R::table_ref())
//         .expr(Expr::asterisk())
//         .and_where(Expr::col(Alias::new("id")).eq(id))
//         .to_owned();
//     one::<R>(&pool, query).await
// }
