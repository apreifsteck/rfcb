use crate::repo::{ChangeError, DBRecord, Insertable, Validatable};
use sea_query::enum_def;
use sea_query::types::Alias;
use sea_query::InsertStatement;
use sqlx::types::chrono::{DateTime, Utc};
use std::hash::Hash;
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

type ID = i32;
#[derive(Debug, sqlx::FromRow)]
pub struct RFCRow {
    pub id: ID,
    #[sqlx(try_from = "String")]
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supersedes: Option<ID>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DBRecord for RFCRow {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}

#[derive(Debug, EnumString, AsRefStr, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
#[enum_def]
pub struct RFCAttrs<'a> {
    proposal: &'a str,
    topic: &'a str,
}

impl<'a> RFCAttrs<'a> {
    pub fn new(topic: &'a str, proposal: &'a str) -> Self {
        Self { topic, proposal }
    }
}

type A = RFCAttrsIden;
impl<'a> Insertable for RFCAttrs<'a> {
    type Output = RFCRow;
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
