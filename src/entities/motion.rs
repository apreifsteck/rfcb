use crate::entities::{participants::Participant, vote::Vote};
use crate::repo::{ChangeError, DBRecord, Insertable, Queryable, Validatable};
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::types::chrono::{DateTime, Utc};
use std::str::FromStr;
use strum::EnumString;
use strum_macros::AsRefStr;

type ID = i32;
#[derive(Debug, sqlx::FromRow)]
#[enum_def]
pub struct Motion {
    pub id: ID,
    pub vote_id: ID,
    pub participant_id: ID,

    #[sqlx(try_from = "String")]
    pub r#type: Type,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, EnumString, AsRefStr, Clone, PartialEq, Eq)]
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

impl From<Type> for sea_query::Value {
    fn from(value: Type) -> sea_query::Value {
        value.as_ref().into()
    }
}

pub struct MotionAttrs<'a> {
    pub vote: &'a Vote,
    pub participant: &'a Participant,
    pub r#type: Type,
    pub comment: Option<&'a str>,
}

impl<'a> Validatable for MotionAttrs<'a> {
    fn validate(&self) -> Result<(), ChangeError> {
        Ok(())
    }
}

impl<'a> DBRecord for Motion {
    fn table_ref() -> Alias {
        Alias::new("motions")
    }
    fn primary_key(&self) -> i32 {
        self.id
    }
}
impl<'a> Insertable for MotionAttrs<'a> {
    type Output = Motion;
    fn inject_values(
        self,
        starter_query: &mut sea_query::InsertStatement,
    ) -> &mut sea_query::InsertStatement {
        let alias = |x| Alias::new(x);
        let columns: Vec<Alias> = ["vote_id", "participant_id", "type", "comment"]
            .into_iter()
            .map(alias)
            .collect();
        starter_query.columns(columns).values_panic([
            self.vote.id.into(),
            self.participant.id.into(),
            self.r#type.into(),
            self.comment.into(),
        ])
    }
}

pub struct MotionQuery<'a> {
    pub vote: Option<&'a Vote>,
    pub participant: Option<&'a Participant>,
    pub r#type: Option<Type>,
}

impl<'a> Queryable for MotionQuery<'a> {
    type Output = Motion;
    fn to_sql(&self) -> String {
        let vote_id: Option<SimpleExpr> = self.vote.map(|x| x.primary_key().into());
        let participant_id: Option<SimpleExpr> = self.participant.map(|x| x.primary_key().into());
        let alias = |x| Alias::new(x);
        let colums_and_values = [
            (alias("vote_id"), vote_id),
            (alias("participant_id"), participant_id),
            (alias("type"), self.r#type.clone().map(|x| x.into())),
        ]
        .into_iter()
        .filter(|(_attr, optional_expr)| optional_expr.is_some())
        .map(|(attr, expr)| (attr, expr.unwrap()));
        let mut query = Query::select()
            .from(Self::Output::table_ref())
            .expr(Expr::asterisk())
            .to_owned();
        colums_and_values
            .fold(&mut query, |acc_query, (col, val)| {
                acc_query.and_where(Expr::col(col).eq(val))
            })
            .to_string(PostgresQueryBuilder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entities::*, repo};
    use sqlx::PgPool;
    #[sqlx::test]
    fn returns_struct_when_valid(pool: PgPool) {
        let rfc = rfc::factory(&pool).await;
        let vote = vote::factory(&pool, rfc.id).await;
        let participant = participants::factory(&pool).await;
        let attrs = MotionAttrs {
            vote: &vote,
            participant: &participant,
            r#type: Type::Accept,
            comment: Some("Hello"),
        };

        let result = repo::insert(&pool, attrs).await.unwrap();
        assert_eq!(result.participant_id, participant.id);
        assert_eq!(result.vote_id, vote.id);
        assert_eq!(result.comment, Some("Hello".to_string()));
        assert_eq!(result.r#type, Type::Accept);
    }
}
