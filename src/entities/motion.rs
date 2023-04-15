use crate::entities::{participants::Participant, vote::Vote};
use crate::repo::{ChangeError, TableRef};
use sea_query::types::Alias;
use sqlx::types::chrono::{DateTime, Utc};
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

#[derive(Debug, sqlx::FromRow)]
pub struct Motion {
    pub id: i32,
    pub vote_id: i32,
    pub participant_id: i32,

    #[sqlx(try_from = "String")]
    pub r#type: Type,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, EnumString, AsRefStr, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Type {
    Accept,
    Reject,
}

impl TryFrom<String> for Type {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_ref()).or(Err("Failed to serialize value to Motion::Type"))
    }
}

impl TableRef for Motion {
    fn table_ref() -> Alias {
        Alias::new("motions")
    }
}

pub struct MotionAttrs<'a> {
    pub vote: &'a Vote,
    pub participant: &'a Participant,
    pub r#type: Type,
    pub comment: &'a str,
}

// pub async fn create_new_motion(&MotionAttrs) -> Result<Motion, ChangeError> {
//     //take motion attrs, upsert into db
//     Result()
// }

// #[cfg(test)]
// mod tests {
//     mod create_new_motion {
//         use sea_query::{Expr, Query};
//         use sqlx::PgPool;
//
//         use crate::entities::rfc;
//
//         use super::super::*;
//         #[sqlx::test]
//         fn returns_struct_when_valid(pool: PgPool) {
//             let rfc = rfc::factory(&pool).await;
//             dbg!(rfc);
//             assert!(false);
//             //Damn I wish I had factories now
//             // let Vote
//             // let attrs = MotionAttrs{}
//         }
//     }
// }
