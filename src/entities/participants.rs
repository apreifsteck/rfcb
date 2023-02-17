use sqlx::postgres::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Error;

#[derive(Debug)]
pub struct Participant {
    pub id: i32,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_one(pool: &PgPool, username: &str) -> Result<Participant, Error> {
    let participant = sqlx::query_as!(
        Participant,
        r#"
        INSERT INTO participants (username) VALUES
        ($1)
        RETURNING *
        "#,
        username
    )
    .fetch_one(pool)
    .await?;

    Ok(participant)
}

#[sqlx::test]
async fn works(pool: PgPool) {
    let participant = create_one(&pool, "Austin").await.unwrap();
    println!("{:?}", participant);
    assert_eq!("Austin", participant.username);
}
