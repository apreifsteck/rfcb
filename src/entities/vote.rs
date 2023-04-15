use crate::repo::{Insertable, TableRef, Validatable};
use chrono::Days;
use sea_query::{enum_def, Alias};
use sqlx::types::chrono::{DateTime, Utc};
use validator::Validate;

type ID = i32;
type UtcDate = DateTime<Utc>;
#[derive(Debug, sqlx::FromRow)]
pub struct Vote {
    pub id: ID,
    pub request_for_comment_id: ID,
    pub deadline: UtcDate,
    pub created_at: UtcDate,
    pub updated_at: UtcDate,
}

#[derive(Debug, Validate)]
#[enum_def]
struct VoteAttrs {
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

impl Insertable for VoteAttrs {
    type Output = Vote;
    fn inject_values<'a>(
        &'a self,
        starter_query: &'a mut sea_query::InsertStatement,
    ) -> &mut sea_query::InsertStatement {
        starter_query
            .columns([VoteAttrsIden::RfcId, VoteAttrsIden::Deadline])
            .values_panic([self.rfc_id.into(), self.deadline.into()])
    }
}

impl Validatable for VoteAttrs {
    fn validate(&self) -> Result<(), crate::repo::ChangeError> {
        Ok(())
    }
}

impl TableRef for VoteAttrs {
    fn table_ref() -> Alias {
        Alias::new("votes")
    }
}

// So here is the question... Does this object recieve an RFC,
// automaticall requiring a trip to the database, or does it just
// accept an ID, and trust whatever came before is secure and gave it good stuff?

use sqlx::PgPool;
pub async fn factory(pool: &PgPool, rfc_id: ID) -> Vote {
    use crate::repo;

    let mut attrs = VoteAttrs {
        rfc_id,
        deadline: None,
    };
    attrs.add_defaults(Days::new(7));
    repo::insert(pool, attrs).await.unwrap()
}

#[cfg(test)]
mod tests {
    use chrono::{Days, Utc};

    use super::VoteAttrs;

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
