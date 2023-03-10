use crate::changeset::{Changeset, TableRef, Valuable};
use sea_query::expr::SimpleExpr;
use sea_query::types::Alias;
use sea_query::Iden;
use sqlx::postgres::{PgPool, PgRow};
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
        Status::from_str(value.as_ref()).or(Err("Something borked in DB"))
    }
}

impl TableRef for RFC {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
}

#[derive(Debug, Iden, Clone, PartialEq, Eq, Hash)]
enum RFCAttrs {
    Status(Status),
    Proposal(String),
    Topic(String),
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

// fn create(proposal: &str, topic: &str) -> Result<RFC> {
//     let cs: Changeset<RFCAttrs, RFC> = Changeset::new(None);
//     //TODO add validations
//     cs.add_change(RFCAttrs::Topic(topic.to_string()), None);
//     cs.add_change(RFCAttrs::Proposal(proposal.to_string()), None);
//     cs.validate()?.insert(&pool).await.unwrap()
// }

// #[cfg(test)]
// mod tests {
//     mod more_test {
//         #[sqlx::test]
//         fn returns_struct_when_valid(pool: PgPool) {
//             let mut cs = setup();
//             cs.validate().unwrap();
//             if let Ok(RFC {
//                 id,
//                 status,
//                 proposal,
//                 topic,
//                 supersedes,
//                 ..
//             }) = cs.insert(&pool).await
//             {
//                 assert!(id > 0);
//                 assert!(status == Status::Active);
//                 assert!(proposal == "Who goes there");
//                 assert!(topic == "the topic");
//                 assert!(supersedes == None);
//             }
//         }
//     }
// }
