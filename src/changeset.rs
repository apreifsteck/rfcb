use sea_query::{Alias, Iden, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Error, Executor};
use std::collections::HashSet;
use std::hash::Hash;

pub trait TableRef {
    fn table_ref() -> Alias;
}
pub trait Valuable {
    fn value(&self) -> SimpleExpr;
}
pub struct Changeset<T, D> {
    changes: HashSet<T>,
    data: Option<D>,
    valid: bool,
}

impl<T, D> Changeset<T, D>
where
    T: Eq + Hash + Iden + Valuable + Clone + 'static,
    for<'a> D: TableRef + sqlx::FromRow<'a, PgRow>,
{
    pub fn new(backing_data: Option<D>) -> Self {
        Changeset {
            changes: HashSet::new(),
            data: backing_data,
            valid: false,
        }
    }
    pub fn add_change(&mut self, change: T) -> &mut Self {
        self.changes.insert(change);
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
    #[test]
    fn will_not_accept_duplicate_change_types() {
        assert!(false)
    }
}
#[cfg(test)]
mod validate_tests {
    #[test]
    fn fails_if_required_attrs_not_present() {
        assert!(false)
    }
    #[test]
    fn passes_if_required_attrs_present() {
        assert!(false)
    }
}

#[cfg(test)]
mod insert_tests {
    use sqlx::PgPool;
    #[sqlx::test]
    fn will_not_insert_unless_valid(_pool: PgPool) {
        assert!(false)
    }
    #[sqlx::test]
    fn renders_back_sane_error_if_borked(_pool: PgPool) {
        assert!(false)
    }
}
