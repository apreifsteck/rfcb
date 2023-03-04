use sea_query::{Alias, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Error, Executor};
use std::mem;
use std::vec::Vec;

pub trait TableRef {
    fn table_ref() -> Alias;
}
pub trait Valuable {
    fn value(&self) -> SimpleExpr;
}
pub struct Changeset<T, D> {
    changes: Vec<T>,
    data: Option<D>,
    valid: bool,
}

impl<T, D> Changeset<T, D>
where
    T: Eq + Iden + Valuable + Clone + 'static + std::fmt::Debug,
    for<'a> D: TableRef + sqlx::FromRow<'a, PgRow>,
{
    pub fn new(backing_data: Option<D>) -> Self {
        Changeset {
            changes: Vec::new(),
            data: backing_data,
            valid: false,
        }
    }
    pub fn add_change(&mut self, change: T) -> &mut Self {
        match self
            .changes
            .iter()
            .find(|&cur_change| mem::discriminant(cur_change) == mem::discriminant(&change))
        {
            Some(_) => panic!("tried to add two of the same type of change"),
            None => self.changes.push(change),
        }
        self
    }
    pub fn validate(&mut self) -> Result<&mut Self, Error> {
        self.valid = true;
        Ok(self)
    }
    pub async fn insert(self, pool: &PgPool) -> Result<D, Error> {
        pool.fetch_one(self.build_query().as_str())
            .await
            .map(|row| D::from_row(&row).unwrap())
    }
    fn build_query(&self) -> String {
        Query::insert()
            .into_table(D::table_ref())
            .columns(self.changes.clone())
            .values_panic(self.changes.clone().iter().map(|x| x.value()))
            .to_string(PostgresQueryBuilder)
    }
}

#[cfg(test)]
mod add_change_tests {
    use super::*;
    use crate::entities::rfc::{RequestForComments, Status, RFC};

    //TODO create a bogus enum and stuff here
    fn setup() -> Changeset<RequestForComments, RFC> {
        let cs: Changeset<RequestForComments, RFC> = Changeset::new(None);
        cs
    }

    #[test]
    #[should_panic]
    fn will_not_accept_duplicate_change_types() {
        let mut cs = setup();
        cs.add_change(RequestForComments::Status(Status::Active));
        cs.add_change(RequestForComments::Status(Status::Active));
    }

    #[test]
    fn will_accept_distinct_change_types() {
        let mut cs = setup();
        cs.add_change(RequestForComments::Status(Status::Active));
        cs.add_change(RequestForComments::Proposal("Whatever".to_string()));

        assert_eq!(
            vec![
                RequestForComments::Status(Status::Active),
                RequestForComments::Proposal("Whatever".to_string())
            ],
            cs.changes
        )
    }
}
#[cfg(test)]
mod validate_tests {
    #[test]
    fn fails_if_required_attrs_not_present() {
        // Desired behaviour
        // Be able to add validations to the changeset.
        // The validations should operate on particular changes
        // Maybe since we're assuming you can only have one change per attribute
        // we could build a map of changes, where the discriminant is the key
        // Validations could be lists to apply to changes.
        // Question: How to pass in an enum variant without instantiating data?
        // Maybe stringify an enum variant? But then you still have to instantiate it
        // OR
        // bundle the validations in along with the change addendum
    }
    #[test]
    fn passes_if_required_attrs_present() {
        assert!(false)
    }
}
//
// #[cfg(test)]
// mod insert_tests {
//     use sqlx::PgPool;
//     #[sqlx::test]
//     fn will_not_insert_unless_valid(_pool: PgPool) {
//         assert!(false)
//     }
//     #[sqlx::test]
//     fn renders_back_sane_error_if_borked(_pool: PgPool) {
//         assert!(false)
//     }
// }
