use diesel::dsl::max;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::{User, Aircraft, Flight, NewFlight};
use crate::schema::{users, aircraft, flights};

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

pub(crate) fn get_latest_flight_number(conn: &PgConnection, user: &User) -> Option<i32> {
    Flight::belonging_to(user)
        .select(max(flights::number))
        .first::<Option<i32>>(conn)
        .expect("Error loading flights")
}

/// Create a new flight.
pub(crate) fn create_flight(conn: &PgConnection, flight: NewFlight) -> Flight {
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
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1);
        assert_eq!(n, None);

        // Single flight, no number
        diesel::insert_into(flights::table)
            .values(NewFlight {
                number: None,
                user_id: ctx.testuser1.id,
                ..Default::default()
            })
            .execute(&*ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1);
        assert_eq!(n, None);

        // Now insert some flights with a flight number
        diesel::insert_into(flights::table)
            .values(vec![
                NewFlight { number: Some(1), user_id: ctx.testuser1.id, ..Default::default() },
                NewFlight { number: Some(-1), user_id: ctx.testuser1.id, ..Default::default() },
                NewFlight { number: Some(7), user_id: ctx.testuser1.id, ..Default::default() },
                NewFlight { number: None, user_id: ctx.testuser1.id, ..Default::default() },
                NewFlight { number: Some(2), user_id: ctx.testuser1.id, ..Default::default() },
            ])
            .execute(&*ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser1);
        assert_eq!(n, Some(7));

        // The user id is properly taken into account
        let n = get_latest_flight_number(&*ctx.force_get_conn(), &ctx.testuser2);
        assert_eq!(n, None);
    }
}
