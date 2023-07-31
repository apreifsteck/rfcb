use crate::repo::{DBRecord, Insertable, Queryable, Validatable};
use chrono::Days;
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder, Query};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

type ID = i32;
type UtcDate = DateTime<Utc>;
#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct VoteRow {
    pub id: ID,
    pub rfc_id: ID,
    pub deadline: UtcDate,
    pub created_at: UtcDate,
    pub updated_at: UtcDate,
}

impl DBRecord for VoteRow {
    fn table_ref() -> Alias {
        Alias::new("votes")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}

#[derive(Debug, Validate)]
#[enum_def]
pub struct VoteAttrs {
    pub rfc_id: ID,
    pub deadline: Option<UtcDate>,
}

impl VoteAttrs {
    pub fn add_defaults(&mut self, default_deadline: Days) -> &mut Self {
        let deadline = self
            .deadline
            .or(Some(Utc::now().checked_add_days(default_deadline).unwrap()));
        self.deadline = deadline;
        self
    }
}

impl Insertable for VoteAttrs {
    type Output = VoteRow;
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
    type Output = VoteRow;
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
