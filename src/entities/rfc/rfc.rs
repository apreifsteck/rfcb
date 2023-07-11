use crate::entities::vote::{Vote, VoteAttrs};
use crate::repo::{self, Association, ChangeError, HasMany};
use derive_getters::Getters;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::data_bindings::{RFCRow, Status};

type ID = i32;
#[derive(Debug, Getters)]
pub struct RFC {
    id: ID,
    status: Status,
    proposal: String,
    topic: String,
    supersedes: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[getter(skip)]
    votes: HasMany<Vote>,
}

impl RFC {
    pub async fn votes(&mut self, pool: &PgPool) -> Result<Option<&[Vote]>, ChangeError> {
        if !self.votes.is_loaded() {
            repo::load(pool, &mut self.votes).await?;
        }
        Ok(self.votes.unwrap())
    }
}
impl From<RFCRow> for RFC {
    fn from(value: RFCRow) -> Self {
        Self {
            id: value.id,
            status: value.status,
            proposal: value.proposal.to_owned(),
            topic: value.topic.to_owned(),
            supersedes: value.supersedes,
            created_at: value.created_at,
            updated_at: value.updated_at,
            votes: HasMany::new(VoteAttrs {
                rfc_id: value.id,
                deadline: None,
            }),
        }
    }
}

#[cfg(test)]
pub async fn factory(pool: &PgPool) -> RFC {
    use fake::faker::lorem::en::*;
    use fake::Fake;

    use crate::entities::rfc::data_bindings::RFCAttrs;

    let words = || Words(3..5).fake::<Vec<String>>().join(" ");
    let proposal = words();
    let topic = words();
    let attrs = RFCAttrs::new(&topic, &proposal);
    repo::insert(pool, attrs).await.unwrap().into()
}
#[cfg(test)]
mod tests {
    mod sanity_check {

        use crate::entities::rfc::data_bindings::RFCAttrs;
        use crate::entities::vote;

        use super::super::*;
        #[sqlx::test]
        fn returns_struct_when_valid(pool: PgPool) {
            let attrs = RFCAttrs::new("the topic", "Who goes there");

            if let RFC {
                status,
                proposal,
                topic,
                id,
                ..
            } = repo::insert(&pool, attrs).await.unwrap().into()
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
