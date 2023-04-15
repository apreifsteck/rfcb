use crate::changeset::{Changeset, Valuable};
use crate::repo::{self, ChangeError, TableRef};
use sea_query::expr::SimpleExpr;
use sea_query::types::Alias;
use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::hash::Hash;
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

type ID = i32;
#[derive(Debug, sqlx::FromRow)]
pub struct RFC {
    pub id: ID,
    #[sqlx(try_from = "String")]
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supersedes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, EnumString, AsRefStr, Clone, PartialEq, Eq, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Approved,
    Denied,
    Discarded,
    Obsolete,
}

impl TryFrom<String> for Status {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Status::from_str(value.as_ref()).or(Err("Failed to serialize value to RFC::Status"))
    }
}

impl TableRef for RFC {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
}

#[derive(Debug, Iden, Clone)]
enum RFCAttrs {
    Status(Status),
    Proposal(String),
    Topic(String),
}

#[derive(Debug, Iden, Clone)]
enum RFCSelectAttrs {
    ID,
}

impl Valuable for RFCAttrs {
    fn value(&self) -> SimpleExpr {
        match self {
            Self::Status(status) => status.as_ref().into(),
            Self::Proposal(p) => p.into(),
            Self::Topic(t) => t.into(),
        }
    }
}

pub async fn create(pool: &PgPool, proposal: &str, topic: &str) -> Result<RFC, ChangeError> {
    let mut cs: Changeset<RFCAttrs, RFC> = Changeset::new(None);
    //TODO add validations
    cs.add_change(RFCAttrs::Topic(topic.to_string()), None);
    cs.add_change(RFCAttrs::Proposal(proposal.to_string()), None);
    cs.add_change(RFCAttrs::Status(Status::Active), None);
    repo::insert(pool, cs).await
}

// TODO
// So I guess we have create done-ish. Maybe we need to create a search wrapper?
// I'm thinking maybe there could be something like a Query type/enum? Maybe theres like an
// IDQuery and a ParameterQuery and these implement maybe a Queryable trait from the repo?

#[cfg(test)]
pub async fn factory(pool: &PgPool) -> RFC {
    use fake::faker::lorem::en::*;
    use fake::{Dummy, Fake, Faker};

    let words = || Words(3..5).fake::<Vec<String>>().join(" ");
    let proposal = words();
    let topic = words();
    create(pool, &proposal, &topic).await.unwrap()
}
mod tests {
    mod create_tests {
        use sea_query::{Expr, Query};

        use super::super::*;
        #[sqlx::test]
        fn returns_struct_when_valid(pool: PgPool) {
            if let Ok(RFC {
                id,
                status,
                proposal,
                topic,
                supersedes,
                ..
            }) = create(&pool, "Who goes there", "the topic").await
            {
                assert!(id > 0);
                assert!(status == Status::Active);
                assert!(proposal == "Who goes there");
                assert!(topic == "the topic");
                assert!(supersedes == None);
            }
        }
    }
}
