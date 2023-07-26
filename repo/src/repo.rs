use super::{assocs::*, attributes::*, error::ChangeError};
use sqlx::{Database, Executor};

pub async fn load<D, E>(
    pool: impl Executor<'_, Database = D>,
    assoc: &mut dyn AssociationLoader<E, D>,
) -> Result<(), ChangeError>
where
    D: Database,
    E: Entity<D>,
{
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

pub async fn insert<R, I, D>(
    pool: impl Executor<'_, Database = D>,
    data: I,
) -> Result<I::Output, ChangeError>
where
    D: Database,
    R: for<'b> sqlx::FromRow<'b, D::Row>,
    I: Insertable<D, Output = R> + Validatable,
{
    data.validate()?;
    let query = data.prepare_query();
    let row = pool.fetch_one(query.as_str()).await?;
    let obj: I::Output = <I as Insertable<D>>::Output::from_row(&row)?;
    Ok(obj)
}

pub async fn all<D, R>(
    pool: impl Executor<'_, Database = D>,
    query: &dyn Queryable<D, Output = R>,
) -> Result<impl IntoIterator<Item = R>, ChangeError>
where
    D: Database,
    R: DBRecord<D>,
{
    let rows = pool.fetch_all(query.to_sql().as_str()).await?;
    //TODO: figure out what to do if a read fails
    let objs: Vec<R> = rows
        .into_iter()
        .map(|row| R::from_row(&row).unwrap())
        .collect();
    Ok(objs)
}

// TODO what happens when there's no matching row?
pub async fn one<D, R>(
    pool: impl Executor<'_, Database = D>,
    query: &dyn Queryable<D, Output = R>,
) -> Result<R, ChangeError>
where
    D: Database,
    R: DBRecord<D>,
{
    let row = pool.fetch_one(query.to_sql().as_str()).await?;
    let obj: R = R::from_row(&row)?;
    Ok(obj)
}
