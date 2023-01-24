use crate::database::DB;
use crate::schema::participants;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Participant {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable)]
#[diesel(table_name = participants)]
pub struct NewParticipant<'a> {
    pub username: &'a str,
}

pub fn insert(db: &impl DB<PgConnection>, username: &str) -> QueryResult<usize> {
    let new_participant = NewParticipant { username };
    let query = diesel::insert_into(participants::table).values(&new_participant);
    db.execute(query)
}

#[cfg(test)]
mod tests {
    //    #[test]
}
