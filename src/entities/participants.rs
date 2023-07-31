use crate::repo::{DBRecord, Insertable, Validatable};
use sea_query::{enum_def, Alias, InsertStatement};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Participant {
    pub id: i32,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[enum_def]
pub struct ParticipantAttrs {
    pub username: String,
}

impl DBRecord for Participant {
    fn table_ref() -> Alias {
        Alias::new("participants")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}
type A = ParticipantAttrsIden;
impl Insertable for ParticipantAttrs {
    type Output = Participant;
    fn inject_values<'a>(self, starter_query: &'a mut InsertStatement) -> &mut InsertStatement {
        starter_query
            .columns([A::Username])
            .values_panic([self.username.into()])
    }
}
impl Validatable for ParticipantAttrs {
    fn validate(&self) -> Result<(), crate::repo::ChangeError> {
        Ok(())
    }
}

#[cfg(test)]
pub async fn factory(pool: &sqlx::PgPool) -> Participant {
    use crate::repo;
    use fake::faker::name::raw::*;
    use fake::locales::*;
    use fake::Fake;
    let attrs = ParticipantAttrs {
        username: Name(EN).fake(),
    };
    repo::insert(pool, attrs).await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::ParticipantAttrs;
    use crate::repo;
    use sqlx::PgPool;
    #[sqlx::test]
    async fn works(pool: PgPool) {
        dbg!("running test");
        let participant = repo::insert(
            &pool,
            ParticipantAttrs {
                username: "Austin".to_string(),
            },
        )
        .await
        .unwrap();

        assert_eq!("Austin", participant.username);
    }
}
