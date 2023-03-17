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
        Ok(row) => R::from_row(&row).map_err(|error| ChangeError::DBError(error)),
        Err(err) => Err(ChangeError::DBError(err)),
    }
}
