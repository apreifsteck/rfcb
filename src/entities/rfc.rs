use sea_query::types::IntoTableRef;
use sea_query::Iden;
use sea_query::Query;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Error, PgPool};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug)]
pub struct RFC {
    pub id: i32,
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supercedes: Option<Box<RFC>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Iden)]
enum Columns {
    Status(Status),
    Proposal(String),
}

#[test]
fn test() {
    assert_eq!(Columns::Status(Status::Active).to_string(), "status");
}

pub struct Changeset<T, D> {
    for_table: &'static str,
    changes: HashSet<T>,
    data: Option<D>,
    valid: bool,
}

// How to turn changeset -> qurey
// How to do custom/generic validations
// How do only enforce one change of a particular type
pub trait storable {
    fn table_name() {}
}

impl<T, D> Changeset<T, D>
where
    T: Eq + Hash + Iden + IntoTableRef,
{
    fn new(table: &str, backing_data: Option<D>) -> Self {
        Changeset {
            for_table: table,
            changes: HashSet::new(),
            data: backing_data,
            valid: false,
        }
    }
    fn add_change(&mut self, change: T) -> &mut Self {
        self.changes.insert(change);
        self
    }
    fn validate(&mut self) -> Result<&mut Self, Error> {
        self.valid = true;
        Ok(self)
    }
    //Changeset::new()
    //.add_change(Proposal("Here is the proposal"))
    //.add_change(Topic("Here is the Topic"))
    //.validate()
    //.insert()
    //

    //Database operations
    //Changeset -> query -> database
    // How to dynamically get the table name?
    fn insert(&self) -> Result<D, Error> {
        let query = Query::insert().into_table("What can I put here");
    }
}

#[derive(Debug)]
pub enum Status {
    Active,
    Approved,
    Denied,
    Discarded,
}

pub enum Mode {
    Insert,
    Update,
}

// #[sqlx::test]
// async fn works(pool: PgPool) {
//     let participant = create_one(&pool, "Austin").await.unwrap();
//     println!("{:?}", participant);
//     assert_eq!("Austin", participant.username);
// }
