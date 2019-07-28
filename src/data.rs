use diesel::{sql_function, PgConnection};
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::sql_types::Text;
use log::error;
use rocket_contrib::database;

use crate::models::{User, Aircraft, Flight, NewFlight};
use crate::schema::{users, aircraft, flights};

sql_function! {
    /// The pgcrypto "crypt" function.
    fn crypt(pw: Text, salt: Text) -> Text;
}

/// Database connection state object.
#[database("flugbuech")]
pub struct Database(diesel::PgConnection);

/// Return the user model with the specified user id.
pub fn get_user(conn: &PgConnection, id: i32) -> Option<User> {
    users::table
        .find(id)
        .first(conn)
        .map_err(|e| { error!("Could not query user: {}", e); e })
        .ok()
}

/// Validate username / password combination. Return the corresponding user model if it is valid.
pub fn validate_login(conn: &PgConnection, username: &str, password: &str) -> Option<User> {
    users::table
        .filter(users::username.eq(username))
        .filter(users::password.eq(crypt(password, users::password)))
        .first::<User>(conn)
        .ok()
}

pub fn get_users(conn: &PgConnection) -> Vec<User> {
    users::table
        .load::<User>(conn)
        .expect("Error loading users")
}

pub fn get_aircraft(conn: &PgConnection) -> Vec<Aircraft> {
    aircraft::table
        .load::<Aircraft>(conn)
        .expect("Error loading aircraft")
}

pub fn get_latest_flight_number(conn: &PgConnection, user: &User) -> Option<i32> {
    Flight::belonging_to(user)
        .select(max(flights::number))
        .first::<Option<i32>>(conn)
        .expect("Error loading flights")
}

/// Create a new flight.
pub fn create_flight(conn: &PgConnection, flight: NewFlight) -> Flight {
    diesel::insert_into(flights::table)
        .values(flight)
        .get_result(conn)
        .expect("Could not create flight")
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[test]
    fn test_get_latest_flight_number() {
        let ctx = test_utils::DbTestContext::new();

        // No flights
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(n, None);

        // Single flight, no number
        diesel::insert_into(flights::table)
            .values(NewFlight {
                number: None,
                user_id: ctx.testuser1.user.id,
                ..Default::default()
            })
            .execute(&*ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(n, None);

        // Now insert some flights with a flight number
        diesel::insert_into(flights::table)
            .values(vec![
                NewFlight { number: Some(1), user_id: ctx.testuser1.user.id, ..Default::default() },
                NewFlight { number: Some(-1), user_id: ctx.testuser1.user.id, ..Default::default() },
                NewFlight { number: Some(7), user_id: ctx.testuser1.user.id, ..Default::default() },
                NewFlight { number: None, user_id: ctx.testuser1.user.id, ..Default::default() },
                NewFlight { number: Some(2), user_id: ctx.testuser1.user.id, ..Default::default() },
            ])
            .execute(&*ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(n, Some(7));

        // The user id is properly taken into account
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser2.user);
        assert_eq!(n, None);
    }

    #[test]
    fn validate_login_invalid_no_user() {
        let ctx = test_utils::DbTestContext::new();
        // No user exists, this must fail
        let user = validate_login(&*ctx.force_get_conn(), "foobar", "bazbong");
        assert!(user.is_none());
    }

    #[test]
    fn validate_login_invalid_bad_password() {
        let ctx = test_utils::DbTestContext::new();
        // Wrong password, this must fail
        let user = validate_login(
            &*ctx.force_get_conn(),
            &ctx.testuser1.user.username,
            "bazbong",
        );
        assert!(user.is_none());
    }

    #[test]
    fn validate_login_invalid_correct_password() {
        let ctx = test_utils::DbTestContext::new();
        // Correct password, this should succeed
        let user = validate_login(
            &*ctx.force_get_conn(),
            &ctx.testuser1.user.username,
            &ctx.testuser1.password,
        );
        assert!(user.is_some());
        assert_eq!(user.unwrap().id, ctx.testuser1.user.id);
    }
}
