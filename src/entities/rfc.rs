use crate::entities::vote::Vote;
use crate::repo::{
    self, Association, ChangeError, DBRecord, Insertable, Multiplicity, Validatable,
};
use sea_query::enum_def;
use sea_query::types::Alias;
use sea_query::InsertStatement;
use sqlx::postgres::PgRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Row};
use std::hash::Hash;
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

use super::vote::VoteAttrs;

type ID = i32;
#[derive(Debug)]
pub struct RFC {
    pub id: ID,
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supersedes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    votes: Association<Vote>,
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

impl RFC {
    pub async fn votes(&mut self, pool: &PgPool) -> Result<Option<&[Vote]>, ChangeError> {
        if !self.votes.is_loaded() {
            repo::load(pool, &mut self.votes).await?;
        }
        Ok(self.votes.unwrap_data_many())
    }
}

impl FromRow<'_, PgRow> for RFC {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let status: String = row.try_get("status")?;
        // let status = Status::try_from(status).or(Err(sqlx::Error::Decode(Box::new("err"))))?;
        let id: ID = row.try_get("id")?;
        let assoc_query = VoteAttrs {
            rfc_id: id,
            deadline: None,
        };
        let assoc = Association::new(assoc_query, Multiplicity::Many);
        Ok(Self {
            id: row.try_get("id")?,
            status: Status::from_str(&status).unwrap(),
            proposal: row.try_get("proposal")?,
            topic: row.try_get("topic")?,
            supersedes: row.try_get("supersedes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            votes: assoc,
        })
    }
}

impl DBRecord for RFC {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}

impl TryFrom<String> for Status {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Status::from_str(value.as_ref()).or(Err("Failed to serialize value to RFC::Status"))
    }
}

#[derive(Debug, Clone)]
#[enum_def]
pub struct RFCAttrs<'a> {
    // status: Status,
    proposal: &'a str,
    topic: &'a str,
}

type A = RFCAttrsIden;
impl<'a> Insertable for RFCAttrs<'a> {
    type Output = RFC;
    fn inject_values<'b>(self, starter_query: &'b mut InsertStatement) -> &mut InsertStatement {
        starter_query
            .columns([A::Proposal, A::Topic])
            .values_panic([self.proposal.into(), self.topic.into()])
    }
}

impl<'a> Validatable for RFCAttrs<'a> {
    fn validate(&self) -> Result<(), ChangeError> {
        Ok(())
    }
}

// TODO
// So I guess we have create done-ish. Maybe we need to create a search wrapper?
// I'm thinking maybe there could be something like a Query type/enum? Maybe theres like an
// IDQuery and a ParameterQuery and these implement maybe a Queryable trait from the repo?

#[cfg(test)]
pub async fn factory(pool: &PgPool) -> RFC {
    use fake::faker::lorem::en::*;
    use fake::Fake;

    let words = || Words(3..5).fake::<Vec<String>>().join(" ");
    let proposal = words();
    let topic = words();
    let attrs = RFCAttrs {
        topic: &topic,
        proposal: &proposal,
    };
    repo::insert(pool, attrs).await.unwrap()
}
#[cfg(test)]
mod tests {
    mod sanity_check {

        use crate::entities::vote;

        use super::super::*;
        #[sqlx::test]
        fn returns_struct_when_valid(pool: PgPool) {
            let attrs = RFCAttrs {
                proposal: "Who goes there",
                topic: "the topic",
            };
            if let Ok(RFC {
                status,
                proposal,
                topic,
                id,
                ..
            }) = repo::insert(&pool, attrs).await
            {
                assert!(id > 0);
                assert!(status == Status::Active);
                assert!(proposal == "Who goes there");
                assert!(topic == "the topic");
            }
        }

        #[sqlx::test]
        fn can_get_votes(pool: PgPool) {
            let mut rfc = factory(&pool).await;
            let v1 = vote::factory(&pool, rfc.id).await;
            let v2 = vote::factory(&pool, rfc.id).await;

            let votes: Vec<_> = rfc
                .votes(&pool)
                .await
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|v| v.id)
                .collect();
            assert_eq!(vec![v1.id, v2.id], votes)
        }
    }
}
