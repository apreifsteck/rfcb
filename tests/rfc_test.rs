use rfcb::entities::*;
use sqlx::PgPool;

#[sqlx::test]
fn can_get_the_votes(pool: PgPool) {
    let mut rfc = rfc::factory(&pool).await;
    let vote = vote::factory(&pool, rfc.id).await;
    if let Ok(Some(votes)) = rfc.votes(&pool).await {
        assert_eq!(&vec![vote], votes)
    } else {
        panic!()
    }
}
