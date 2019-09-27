use diesel::dsl::max;
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::{sql_function, PgConnection};
use log::error;
use rocket_contrib::database;

use crate::models::{Aircraft, Flight, Location, NewFlight, NewLocation, User};
use crate::schema::{aircraft, flights, locations, users};

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
        .map_err(|e| {
            error!("Could not query user: {}", e);
            e
        })
        .ok()
}

/// Validate username / password combination. Return the corresponding user model if it is valid.
pub fn validate_login(conn: &PgConnection, username: &str, password: &str) -> Option<User> {
    users::table
        .filter(users::username.eq(username))
        .filter(users::password.eq(crypt(password, users::password)))
        .first(conn)
        .ok()
}

pub fn get_users(conn: &PgConnection) -> Vec<User> {
    users::table.load(conn).expect("Error loading users")
}

pub fn get_aircraft(conn: &PgConnection) -> Vec<Aircraft> {
    aircraft::table.load(conn).expect("Error loading aircraft")
}

pub fn get_aircraft_for_user(conn: &PgConnection, user: &User) -> Vec<Aircraft> {
    Aircraft::belonging_to(user)
        .load(conn)
        .expect("Error loading aircraft")
}

pub fn get_aircraft_with_id(conn: &PgConnection, id: i32) -> Option<Aircraft> {
    aircraft::table.find(id)
        .first(conn)
        .optional()
        .expect("Error loading aircraft by id")
}

pub fn get_latest_flight_number(conn: &PgConnection, user: &User) -> Option<i32> {
    Flight::belonging_to(user)
        .select(max(flights::number))
        .first(conn)
        .expect("Error loading flight")
}

/// Create a new flight.
pub fn create_flight(conn: &PgConnection, flight: &NewFlight) -> Flight {
    diesel::insert_into(flights::table)
        .values(flight)
        .get_result(conn)
        .expect("Could not create flight")
}

/// Retrieve all flights of a specific user.
pub fn get_flights_for_user(conn: &PgConnection, user: &User) -> Vec<Flight> {
    Flight::belonging_to(user)
        .order((flights::number.desc(), flights::launch_time.desc()))
        .load(conn)
        .expect("Error loading flights")
}

/// Retrieve flight with the specified ID.
pub fn get_flight_with_id(conn: &PgConnection, id: i32) -> Option<Flight> {
    flights::table.find(id)
        .first(conn)
        .optional()
        .expect("Error loading flight by id")
}

/// Retrieve all locations with the specified IDs.
pub fn get_locations_with_ids(conn: &PgConnection, ids: &[i32]) -> Vec<Location> {
    locations::table
        .filter(locations::id.eq_any(ids))
        .load(conn)
        .expect("Error loading locations")
}

/// Retrieve all locations for the specified user.
pub fn get_locations_for_user(conn: &PgConnection, user: &User) -> Vec<Location> {
    Location::belonging_to(user)
        .order(locations::name)
        .load(conn)
        .expect("Error loading locations")
}

/// Retrieve location with the specified ID.
pub fn get_location_with_id(conn: &PgConnection, id: i32) -> Option<Location> {
    locations::table.find(id)
        .first(conn)
        .optional()
        .expect("Error loading location by id")
}

/// Create a new location.
pub fn create_location(conn: &PgConnection, location: NewLocation) -> Location {
    diesel::insert_into(locations::table)
        .values(location)
        .get_result(conn)
        .expect("Could not create location")
}

/// Save an updated location in the database.
pub fn update_location(conn: &PgConnection, location: &Location) {
    diesel::update(location)
        .set(location)
        .execute(conn)
        .expect("Could not update location");
}

/// Create a new flight.
pub fn update_user_last_aircraft(conn: &PgConnection, user: &User, aircraft_id: i32) {
    diesel::update(user)
        .set(users::last_aircraft_id.eq(aircraft_id))
        .execute(conn)
        .expect("Could not set user last aircraft id");
}

#[cfg(test)]
mod tests {
    use crate::models::NewAircraft;
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
                NewFlight {
                    number: Some(1),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    number: Some(-1),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    number: Some(7),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    number: None,
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    number: Some(2),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
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
    fn test_get_aircraft_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No aircraft
        let a = get_aircraft_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(a.len(), 0);

        // Create some aircraft
        diesel::insert_into(aircraft::table)
            .values(vec![
                NewAircraft {
                    user_id: ctx.testuser1.user.id,
                    model: "Epsilon 8 23".into(),
                    manufacturer: "Advance".into(),
                },
                NewAircraft {
                    user_id: ctx.testuser1.user.id,
                    model: "Green S".into(),
                    manufacturer: "Team5".into(),
                },
                NewAircraft {
                    user_id: ctx.testuser2.user.id,
                    model: "Pi 2".into(),
                    manufacturer: "Advance".into(),
                },
            ])
            .execute(&*ctx.force_get_conn())
            .expect("Could not create aircraft");

        let a1 = get_aircraft_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        let a2 = get_aircraft_for_user(&*ctx.force_get_conn(), &ctx.testuser2.user);
        assert_eq!(
            a1.iter().map(|a| a.model.clone()).collect::<Vec<_>>(),
            vec!["Epsilon 8 23".to_string(), "Green S".to_string()],
        );
        assert_eq!(
            a2.iter().map(|a| a.model.clone()).collect::<Vec<_>>(),
            vec!["Pi 2".to_string()],
        );
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
        let user = validate_login(&*ctx.force_get_conn(), &ctx.testuser1.user.username, "bazbong");
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
