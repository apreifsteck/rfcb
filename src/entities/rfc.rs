use crate::changeset::{TableRef, Valuable};
use sea_query::expr::SimpleExpr;
use sea_query::types::Alias;
use sea_query::Iden;
use sqlx::types::chrono::{DateTime, Utc};
use std::hash::Hash;
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

#[derive(Debug, sqlx::FromRow)]
pub struct RFC {
    pub id: i32,
    #[sqlx(try_from = "String")]
    pub status: Status,
    pub proposal: String,
    pub topic: String,
    pub supercedes: i32,
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
}

impl TryFrom<String> for Status {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Status::from_str(value.as_ref()).or(Err("Something borked in DB"))
    }
}

impl TableRef for RFC {
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
