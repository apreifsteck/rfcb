use crate::entities::rfc;
use crate::repo::{self, ChangeError, TableRef};
use chrono::Duration;
use sea_query::{Alias, Iden};
use sqlx::types::chrono::{DateTime, Utc};

type ID = i32;
#[derive(Debug, sqlx::FromRow)]
pub struct Vote {
    pub id: i32,
    pub request_for_comment_id: ID,
    pub deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Iden, Clone)]
enum VoteAttrs {
    RFCID(ID),
    Deadline(DateTime<Utc>),
}

pub struct()

// So here is the question... Does this object recieve an RFC,
// automaticall requiring a trip to the database, or does it just
// accept an ID, and trust whatever came before is secure and gave it good stuff?
const DEFAULT_VOTE_DURATION: Duration = Duration::days(7);

impl Vote {
    pub async fn create(attrs: VoteAttrs) -> Vote {
        let deadline = attrs.deadline.or(Some(
            Utc::now()
                .checked_add_signed(DEFAULT_VOTE_DURATION)
                .unwrap(),
        ));
    }
}

impl TableRef for Vote {
    fn table_ref() -> Alias {
        Alias::new("request_for_comments")
    }
}
