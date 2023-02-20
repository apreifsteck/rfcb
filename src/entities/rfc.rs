use sea_query::expr::SimpleExpr;
use sea_query::query::Query;
use sea_query::types::Alias;
use sea_query::{Iden, PostgresQueryBuilder};
use sqlx::postgres::PgRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Error, Executor, FromRow, PgPool, Postgres, QueryBuilder};
use std::collections::HashSet;
use std::hash::Hash;
use strum_macros::AsRefStr;

#[derive(Debug, sqlx::FromRow)]
pub struct RFC {
    pub id: i32,
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supercedes: Option<Box<RFC>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(AsRefStr, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    #[strum(serialize = "active")]
    Active,
    #[strum(serialize = "approved")]
    Approved,
    #[strum(serialize = "denied")]
    Denied,
    #[strum(serialize = "discarded")]
    Discarded,
}
pub trait Referenceable {
    fn table_ref() -> Alias;
}
pub trait Valuable {
    fn value(&self) -> SimpleExpr;
}

impl Referenceable for RFC {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
}

#[derive(Iden, Clone, PartialEq, Eq, Hash)]
enum RequestForComments {
    Status(Status),
    Proposal(String),
}

impl Valuable for RequestForComments {
    fn value(&self) -> SimpleExpr {
        match self {
            Self::Status(status) => status.as_ref().into(),
            Self::Proposal(p) => p.into(),
        }
    }
}

pub struct Changeset<T, D> {
    changes: HashSet<T>,
    data: Option<D>,
    valid: bool,
}

// How to turn changeset -> qurey
// How to do custom/generic validations
// How do only enforce one change of a particular type

impl<T, D> Changeset<T, D>
where
    T: Eq + Hash + Iden + Valuable + Clone + 'static,
    for<'a> D: Referenceable + sqlx::FromRow<'a, PgRow>,
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

#[test]
fn build_query() {
    let mut cs: Changeset<RequestForComments, RFC> = Changeset::new(None);
    // Either make order matter or test this indirectly
    cs.add_change(RequestForComments::Status(Status::Active))
        .add_change(RequestForComments::Proposal(String::from("Hello")));
}

pub enum Mode {
    Insert,
    Update,
}
