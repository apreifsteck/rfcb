use crate::{
    api_error::APIError,
    repo::{self, DBRecord, Insertable, Queryable, Validatable},
};
use chrono::Days;
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder, Query};
use sqlx::types::chrono::{DateTime, Utc};
use validator::Validate;

type ID = i32;
type UtcDate = DateTime<Utc>;
#[derive(Debug, sqlx::FromRow, PartialEq, Eq)]
pub struct Vote {
    pub id: ID,
    pub rfc_id: ID,
    pub deadline: UtcDate,
    pub created_at: UtcDate,
    pub updated_at: UtcDate,
}

pub async fn make_new_motion<'a>(
    pool: &sqlx::PgPool,
    motion: MotionAttrs<'a>,
) -> Result<(), APIError<'a>> {
    // types of errors:
    // - DB Error
    // - Vote could have already been finished
    // if deadline is passed, return vote error
    // Else upsert to database
    if motion.vote.is_time_past_deadline(Utc::now()) {
        VoteError {
            vote: motion.vote,
            participant: motion.participant,
            message: format!(
                "Attempted to pass a motion for a vote at {} when deadline is {}",
                Utc::now(),
                motion.vote.deadline
            ),
        }
        .into()
    } else {
        Ok(())
    }?;
    repo::insert(pool, motion).await?;
    Ok(())
}
impl Vote {
    fn is_time_past_deadline(&self, time: UtcDate) -> bool {
        time > self.deadline
    }
}

#[derive(Debug, Validate)]
#[enum_def]
pub struct VoteAttrs {
    pub rfc_id: ID,
    pub deadline: Option<UtcDate>,
}

impl VoteAttrs {
    fn add_defaults(&mut self, default_deadline: Days) -> &mut Self {
        let deadline = self
            .deadline
            .or(Some(Utc::now().checked_add_days(default_deadline).unwrap()));
        self.deadline = deadline;
        self
    }
}

impl DBRecord for Vote {
    fn table_ref() -> Alias {
        Alias::new("votes")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}

impl Insertable for VoteAttrs {
    type Output = Vote;
    fn inject_values<'a>(
        self,
        starter_query: &'a mut sea_query::InsertStatement,
    ) -> &mut sea_query::InsertStatement {
        starter_query
            .columns([VoteAttrsIden::RfcId, VoteAttrsIden::Deadline])
            .values_panic([self.rfc_id.into(), self.deadline.into()])
    }
}

impl Queryable for VoteAttrs {
    type Output = Vote;
    fn to_sql(&self) -> String {
        Query::select()
            .from(Self::Output::table_ref())
            .expr(Expr::asterisk())
            .and_where(Expr::col(VoteAttrsIden::RfcId).eq(self.rfc_id))
            .to_string(PostgresQueryBuilder)
    }
}

impl Validatable for VoteAttrs {
    fn validate(&self) -> Result<(), crate::repo::ChangeError> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VoteError<'a> {
    pub message: String,
    pub vote: &'a Vote,
    pub participant: &'a Participant,
}

impl<'a> From<VoteError<'a>> for Result<(), VoteError<'a>> {
    fn from(value: VoteError<'a>) -> Self {
        Err(value)
    }
}

// So here is the question... Does this object recieve an RFC,
// automaticall requiring a trip to the database, or does it just
// accept an ID, and trust whatever came before is secure and gave it good stuff?

use super::{motion::MotionAttrs, participants::Participant};
#[cfg(test)]
pub async fn factory(pool: &sqlx::PgPool, rfc_id: ID) -> Vote {
    let mut attrs = VoteAttrs {
        rfc_id,
        deadline: None,
    };
    attrs.add_defaults(Days::new(7));
    repo::insert(pool, attrs).await.unwrap()
}

#[cfg(test)]
mod tests {
    mod add_defaults_tests {
        use chrono::{Days, Utc};

        use super::super::VoteAttrs;

        #[test]
        fn add_defaults_adds_deadline_when_not_supplied() {
            let mut attrs = VoteAttrs {
                rfc_id: -1,
                deadline: None,
            };
            attrs.add_defaults(Days::new(7));

            let seven_days_from_now = Utc::now()
                .checked_add_days(Days::new(7))
                .unwrap()
                .date_naive();
            assert_eq!(seven_days_from_now, attrs.deadline.unwrap().date_naive())
        }

        #[test]
        fn does_not_add_default_deadline_when_its_supplied() {
            let mut attrs = VoteAttrs {
                rfc_id: -1,
                deadline: Some(Utc::now().checked_add_days(Days::new(3)).unwrap()),
            };
            attrs.add_defaults(Days::new(7));

            let seven_days_from_now = Utc::now()
                .checked_add_days(Days::new(7))
                .unwrap()
                .date_naive();

            assert_eq!(
                seven_days_from_now,
                attrs.deadline.unwrap().date_naive() + Days::new(4)
            )
        }
    }

    mod make_new_motion_tests {
        use chrono::{Days, Utc};

        use super::super::{Vote, VoteError};
        use crate::{
            api_error::APIError,
            entities::{
                motion::{MotionAttrs, MotionQuery, Type},
                participants::{self, Participant},
                rfc, vote,
            },
            repo,
        };
        fn participant() -> Participant {
            let now = Utc::now();
            Participant {
                id: -1,
                username: "austin".to_string(),
                created_at: now,
                updated_at: now,
            }
        }
        fn vote() -> Vote {
            let now = Utc::now();
            Vote {
                id: -1,
                rfc_id: -1,
                deadline: now,
                created_at: now,
                updated_at: now,
            }
        }

        #[sqlx::test]
        fn returns_error_if_current_date_after_deadline(pool: sqlx::PgPool) {
            use regex::Regex;
            let one_day_ago = Utc::now().checked_sub_days(Days::new(1)).unwrap();
            let mut vote = vote();
            vote.deadline = one_day_ago;
            let participant = participant();
            let motion_attrs = MotionAttrs {
                vote: &vote,
                participant: &participant,
                r#type: Type::Accept,
                comment: None,
            };
            if let APIError::VoteError(result) = vote::make_new_motion(&pool, motion_attrs)
                .await
                .unwrap_err()
            {
                let expected_error = VoteError {
                    participant: &participant,
                    vote: &vote,
                    message: format!(
                        "Attempted to pass motion for a vote at {} when deadline is {}",
                        Utc::now(),
                        vote.deadline
                    ),
                };
                assert_eq!(expected_error.participant.id, result.participant.id);
                assert_eq!(expected_error.vote.id, result.vote.id);
                let date_regex = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{6} UTC";
                let message_regex = Regex::new(
                    (r"Attempted to pass motion for a vote at ".to_owned()
                        + date_regex
                        + " when deadline is "
                        + date_regex)
                        .as_str(),
                )
                .unwrap();
                assert!(message_regex.is_match(expected_error.message.as_str()))
            } else {
                panic!()
            }
        }
        #[sqlx::test]
        fn returns_ok_if_current_date_befor_deadline(pool: sqlx::PgPool) {
            let rfc = rfc::factory(&pool).await;
            let mut vote = vote::factory(&pool, rfc.id).await;
            let participant = participants::factory(&pool).await;

            let one_day_from_now = Utc::now().checked_add_days(Days::new(1)).unwrap();
            vote.deadline = one_day_from_now;

            let motion_attrs = MotionAttrs {
                vote: &vote,
                participant: &participant,
                r#type: Type::Accept,
                comment: None,
            };
            assert!(vote::make_new_motion(&pool, motion_attrs).await.is_ok());

            let created_motion = repo::one(
                &pool,
                &MotionQuery {
                    vote: Some(&vote),
                    participant: Some(&participant),
                    r#type: None,
                },
            )
            .await
            .unwrap();
            assert_eq!(created_motion.participant_id, participant.id);
            assert_eq!(created_motion.vote_id, vote.id);
        }
    }
}
