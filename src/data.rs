use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::{User, Aircraft};
use crate::schema::{users, aircraft};

pub(crate) fn get_users(conn: &PgConnection) -> Vec<User> {
    users::table
        .load::<User>(conn)
        .expect("Error loading users")
}

pub(crate) fn get_aircraft(conn: &PgConnection) -> Vec<Aircraft> {
    aircraft::table
        .load::<Aircraft>(conn)
        .expect("Error loading aircraft")
}
