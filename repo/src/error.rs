use std::fmt::Debug;
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
