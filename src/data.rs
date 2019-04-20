use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::*;
use crate::schema::users;

pub(crate) fn get_users(conn: &PgConnection) -> Vec<User> {
    users::table
        .load::<User>(conn)
        .expect("Error loading users")
}
