use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::*;
use crate::schema::users::dsl::*;

pub(crate) fn get_users(conn: &PgConnection) -> Vec<User> {
    users
        .load::<User>(conn)
        .expect("Error loading users")
}
