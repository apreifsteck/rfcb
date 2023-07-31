use super::data_bindings::VoteRow;
use crate::{
    api_error::APIError,
    entities::motion::{Motion, MotionAssocQuery, MotionAttrs},
    entities::participants::Participant,
    repo::{self, Entity, HasMany},
};
use derive_getters::Getters;
use sqlx::types::chrono::{DateTime, Utc};

type ID = i32;
type UtcDate = DateTime<Utc>;
#[derive(Debug, Getters, PartialEq, Eq)]
pub struct Vote {
    id: ID,
    rfc_id: ID,
    deadline: UtcDate,
    created_at: UtcDate,
    updated_at: UtcDate,
    #[getter(skip)]
    motions: HasMany<Motion>,
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

impl Entity for Vote {
    type Record = VoteRow;
}

impl From<VoteRow> for Vote {
    fn from(value: VoteRow) -> Self {
        Self {
            id: value.id,
            rfc_id: value.rfc_id,
            deadline: value.deadline,
            created_at: value.created_at,
            updated_at: value.updated_at,
            motions: HasMany::new(MotionAssocQuery { vote_id: value.id }),
        }
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

#[cfg(test)]
use super::data_bindings::VoteAttrs;
#[cfg(test)]
use chrono::Days;
#[cfg(test)]
pub async fn factory(pool: &sqlx::PgPool, rfc_id: ID) -> Vote {
    let mut attrs = VoteAttrs {
        rfc_id,
        deadline: None,
    };
    attrs.add_defaults(Days::new(7));
    let res = repo::insert(pool, attrs).await.unwrap();
    res.into()
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
                motion::{MotionAssocQuery, MotionAttrs, MotionQuery, Type},
                participants::{self, Participant},
                rfc, vote,
            },
            repo::{self, HasMany},
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
                motions: HasMany::new(MotionAssocQuery { vote_id: -1 }),
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
            if let APIError::VoteError(result) = vote::entity::make_new_motion(&pool, motion_attrs)
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
            let rfc = rfc::entity::factory(&pool).await;
            dbg!(&rfc);
            let mut vote = vote::entity::factory(&pool, rfc.id().to_owned()).await;
            dbg!(&vote);
            let participant = participants::factory(&pool).await;

            let one_day_from_now = Utc::now().checked_add_days(Days::new(1)).unwrap();
            vote.deadline = one_day_from_now;

            let motion_attrs = MotionAttrs {
                vote: &vote,
                participant: &participant,
                r#type: Type::Accept,
                comment: None,
            };
            assert!(vote::entity::make_new_motion(&pool, motion_attrs)
                .await
                .is_ok());

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
