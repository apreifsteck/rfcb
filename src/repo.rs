use std::fmt::Debug;

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

#[derive(Debug)]
enum Loadable<T> {
    Loaded(Data<T>),
    NotLoaded,
}

impl<T> Loadable<T> {
    fn unwrap_to_inner_ref(&self) -> &Data<T> {
        if let Loadable::Loaded(t) = self {
            &t
        } else {
            panic!()
        }
    }
}
//
// Do I need both data and multiplicity?
#[derive(Clone, Copy, Debug)]
pub enum Multiplicity {
    Many,
    One,
}
//
#[derive(Debug)]
enum Data<T> {
    None,
    One(T),
    Many(Vec<T>),
}

impl<T> Data<T> {
    fn unwrap_option_many(&self) -> Option<&[T]> {
        match self {
            Self::None => None,
            Self::Many(ref d) => Some(d),
            _ => panic!(),
        }
    }
    fn unwrap_option_one(&self) -> Option<&T> {
        match self {
            Self::None => None,
            Self::One(ref d) => Some(d),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct Association<T: DBRecord> {
    data: Loadable<T>,
    load_query: Box<dyn Queryable<Output = T>>,
    multiplicity: Multiplicity,
}

impl<T: DBRecord> Association<T> {
    pub fn new(
        load_query: impl Queryable<Output = T> + 'static,
        multiplicity: Multiplicity,
    ) -> Self {
        Self {
            data: Loadable::NotLoaded,
            load_query: Box::new(load_query),
            multiplicity,
        }
    }
    pub fn query(&self) -> &dyn Queryable<Output = T> {
        self.load_query.as_ref()
    }
    pub fn is_loaded(&self) -> bool {
        match self.data {
            Loadable::NotLoaded => false,
            Loadable::Loaded(_) => true,
        }
    }
    pub fn multiplicity(&self) -> Multiplicity {
        self.multiplicity
    }
    pub fn unwrap_data_many(&self) -> Option<&[T]> {
        self.data.unwrap_to_inner_ref().unwrap_option_many()
    }
    pub fn unwrap_data_one(&self) -> Option<&T> {
        self.data.unwrap_to_inner_ref().unwrap_option_one()
    }
}
pub async fn load<D: DBRecord>(
    pool: &PgPool,
    assoc: &mut Association<D>,
) -> Result<(), ChangeError> {
    match assoc.multiplicity() {
        Multiplicity::One => {
            let res: D = one(pool, assoc.query()).await?;
            assoc.data = Loadable::Loaded(Data::One(res));
            Ok(())
        }
        Multiplicity::Many => {
            let res = all(pool, assoc.query()).await?;
            assoc.data = Loadable::Loaded(Data::Many(res.into_iter().collect()));
            Ok(())
        }
        _ => Ok(()),
    }
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
pub trait Queryable: Debug {
    type Output: DBRecord;
    fn to_sql(&self) -> String;
}

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

pub async fn all<D>(
    pool: &PgPool,
    query: &dyn Queryable<Output = D>,
) -> Result<impl IntoIterator<Item = D>, ChangeError>
where
    D: DBRecord,
{
    let rows = pool.fetch_all(query.to_sql().as_str()).await?;
    //TODO: figure out what to do if a read fails
    let objs: Vec<D> = rows
        .into_iter()
        .map(|row| D::from_row(&row).unwrap())
        .collect();
    Ok(objs)
}

// TODO what happens when there's no matching row?
pub async fn one<D>(pool: &PgPool, query: &dyn Queryable<Output = D>) -> Result<D, ChangeError>
where
    D: DBRecord,
{
    let row = pool.fetch_one(query.to_sql().as_str()).await?;
    let obj: D = D::from_row(&row)?;
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
