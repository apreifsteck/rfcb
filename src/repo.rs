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

#[derive(Debug, PartialEq, Eq)]
pub enum Loadable<T> {
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
#[derive(Debug, PartialEq, Eq)]
pub enum Data<T> {
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

pub trait AssociationLoader<T: Entity> {
    fn query(&self) -> &dyn Queryable<Output = T::Record>;
    fn multiplicity(&self) -> Multiplicity;
    fn load(&mut self, data: Loadable<T>);
}

pub trait Association<'a, T: Entity + 'a>: AssociationLoader<T> + PartialEq + Eq {
    type Unwrapped;
    fn is_loaded(&self) -> bool;
    fn unwrap(&'a self) -> Self::Unwrapped;
}

#[derive(Debug)]
pub struct HasMany<T>
where
    T: Entity,
{
    data: Loadable<T>,
    load_query: Box<dyn Queryable<Output = T::Record>>,
}

impl<T: Entity> AssociationLoader<T> for HasMany<T> {
    fn query(&self) -> &dyn Queryable<Output = T::Record> {
        self.load_query.as_ref()
    }

    fn multiplicity(&self) -> Multiplicity {
        Multiplicity::Many
    }
    fn load(&mut self, data: Loadable<T>) {
        self.data = data;
    }
}

impl<'a, T: Entity + PartialEq + Eq + 'a> Association<'a, T> for HasMany<T> {
    type Unwrapped = Option<&'a [T]>;
    fn unwrap(&'a self) -> Self::Unwrapped {
        self.data.unwrap_to_inner_ref().unwrap_option_many()
    }

    fn is_loaded(&self) -> bool {
        match self.data {
            Loadable::NotLoaded => false,
            Loadable::Loaded(_) => true,
        }
    }
}

impl<T: Entity> HasMany<T> {
    pub fn new(load_query: impl Queryable<Output = T::Record> + 'static) -> Self {
        Self {
            data: Loadable::NotLoaded,
            load_query: Box::new(load_query),
        }
    }
}
//
impl<T> PartialEq for HasMany<T>
where
    T: Entity + PartialEq + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<T: Entity> Eq for HasMany<T> where T: Eq {}

pub async fn load<E: Entity>(
    pool: &PgPool,
    assoc: &mut dyn AssociationLoader<E>,
) -> Result<(), ChangeError> {
    match assoc.multiplicity() {
        Multiplicity::One => {
            let res: E::Record = one(pool, assoc.query()).await?;
            assoc.load(Loadable::Loaded(Data::One(res.into())));
            Ok(())
        }
        Multiplicity::Many => {
            let res = all(pool, assoc.query()).await?;
            assoc.load(Loadable::Loaded(Data::Many(
                res.into_iter().map(|i| i.into()).collect(),
            )));
            Ok(())
        }
    }
}

pub trait Entity: Sized {
    type Record: DBRecord + Into<Self>;
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
    let query = data.prepare_query();
    let row = pool.fetch_one(query.as_str()).await?;
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
