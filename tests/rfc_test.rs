use rfcb::entities::*;
use sqlx::PgPool;

// #[sqlx::test]
// fn can_get_the_votes(pool: PgPool) {
//     let rfc = rfc::factory(&pool).await;
//     let vote = vote::factory(&pool, rfc.id).await;
//     assert_eq!(vec![vote], rfc.votes())
// }
