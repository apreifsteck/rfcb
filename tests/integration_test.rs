use rfcb::database::{self, DB};
use rfcb::entites::participants;
#[test]
fn how_the_heck_do_dbs_work() {
    //create the db
    let db = database::TestDB::init();
    let affected = participants::insert(&db, "Austin").unwrap();
    //inspect the result
    println!("affected: {:?}", &affected);
    assert_eq!(1, affected);
}
