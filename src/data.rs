use std::env;
use std::io;

use diesel::dsl::{count, max};
use diesel::prelude::*;
use diesel::result::QueryResult;
use diesel::sql_types::{Double, Integer, Text};
use diesel::{sql_function, sql_query, PgConnection};
use diesel_geography::sql_types::Geography;
use diesel_geography::types::GeogPoint;
use log::error;
use rocket_contrib::database;

use crate::models::{Flight, Glider, Location, LocationWithDistance, User};
use crate::models::{NewFlight, NewGlider, NewLocation};
use crate::schema::{flights, gliders, locations, users};

sql_function! {
    /// The pgcrypto "crypt" function.
    fn crypt(pw: Text, salt: Text) -> Text;
}

sql_function! {
    /// The pgcrypto "gen_salt" function.
    fn gen_salt(type_: Text, iter_count: Integer) -> Text;
}

const PW_SALT_ITERATIONS: i32 = 10;

/// Database connection state object.
#[database("flugbuech")]
pub struct Database(diesel::PgConnection);

embed_migrations!();

/// Run migrations on the database indicated with `DATABASE_URL`.
pub fn run_migrations() -> Result<(), String> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    match PgConnection::establish(&database_url) {
        Ok(connection) => embedded_migrations::run_with_output(&connection, &mut io::stdout())
            .map_err(|e| format!("Could not run migrations: {}", e)),
        Err(e) => Err(format!("Could not connect to database: {}", e)),
    }
}

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

pub fn get_user_count(conn: &PgConnection) -> i64 {
    users::table
        .select(count(users::id))
        .first(conn)
        .expect("Error loading user count")
}

/// Create a user in the database. The password will be hashed.
pub fn create_user(
    conn: &PgConnection,
    username: impl Into<String>,
    password: impl Into<String>,
    email: impl Into<String>,
) -> User {
    diesel::insert_into(users::table)
        .values(&(
            users::username.eq(username.into()),
            users::password.eq(crypt(password.into(), gen_salt("bf", PW_SALT_ITERATIONS))),
            users::email.eq(email.into()),
        ))
        .get_result(conn)
        .expect("Could not create user")
}

pub fn get_gliders(conn: &PgConnection) -> Vec<Glider> {
    gliders::table.load(conn).expect("Error loading gliders")
}

pub fn get_glider_count(conn: &PgConnection) -> i64 {
    gliders::table
        .select(count(gliders::id))
        .first(conn)
        .expect("Error loading glider count")
}

pub fn get_gliders_for_user(conn: &PgConnection, user: &User) -> Vec<Glider> {
    Glider::belonging_to(user)
        .load(conn)
        .expect("Error loading gliders")
}

pub fn get_glider_with_id(conn: &PgConnection, id: i32) -> Option<Glider> {
    gliders::table
        .find(id)
        .first(conn)
        .optional()
        .expect("Error loading glider by id")
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

/// Save an updated flight in the database.
pub fn update_flight(conn: &PgConnection, flight: &Flight) {
    diesel::update(flight)
        .set(flight)
        .execute(conn)
        .expect("Could not update flight");
}

pub fn get_flight_count(conn: &PgConnection) -> i64 {
    flights::table
        .select(count(flights::id))
        .first(conn)
        .expect("Error loading flight count")
}

/// Retrieve all flights of a specific user.
pub fn get_flights_for_user(conn: &PgConnection, user: &User) -> Vec<Flight> {
    Flight::belonging_to(user)
        .order((flights::number.desc(), flights::launch_time.desc()))
        .load(conn)
        .expect("Error loading flights")
}

/// Retrieve the highest flight number for a specific user.
pub fn get_max_flight_number_for_user(conn: &PgConnection, user: &User) -> Option<i32> {
    Flight::belonging_to(user)
        .select(max(flights::number))
        .first(conn)
        .expect("Error loading flight count")
}

/// Retrieve flight with the specified ID.
pub fn get_flight_with_id(conn: &PgConnection, id: i32) -> Option<Flight> {
    flights::table
        .find(id)
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

/// Retrieve all locations for the specified user within a specified radius
/// from the specified coordinates.
pub fn get_locations_around_point(
    conn: &PgConnection,
    user: &User,
    lat: f64,
    lon: f64,
    max_distance_meters: f64,
) -> Vec<LocationWithDistance> {
    let point = GeogPoint {
        x: lon,
        y: lat,
        srid: None,
    };
    sql_query(
        "SELECT id, name, country, elevation, user_id, geog, ST_Distance($1, geog) AS distance
           FROM locations
          WHERE user_id = $2
            AND ST_DWithin(geog, $1, $3)
          ORDER BY distance ASC",
    )
    .bind::<Geography, _>(point)
    .bind::<Integer, _>(user.id)
    .bind::<Double, _>(max_distance_meters)
    .load(conn)
    .expect("Error loading locations")
}

/// Retrieve location with the specified ID.
pub fn get_location_with_id(conn: &PgConnection, id: i32) -> Option<Location> {
    locations::table
        .find(id)
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

/// Create a new glider.
pub fn create_glider(conn: &PgConnection, glider: NewGlider) -> QueryResult<Glider> {
    diesel::insert_into(gliders::table)
        .values(glider)
        .get_result(conn)
}

/// Save an updated glider in the database.
pub fn update_glider(conn: &PgConnection, glider: &Glider) {
    diesel::update(glider)
        .set(glider)
        .execute(conn)
        .expect("Could not update glider");
}

/// Update the "last glider" of a user.
pub fn update_user_last_glider(conn: &PgConnection, user: &User, glider_id: i32) {
    diesel::update(user)
        .set(users::last_glider_id.eq(glider_id))
        .execute(conn)
        .expect("Could not set user last glider id");
}

#[cfg(test)]
mod tests {
    use crate::models::NewGlider;
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
    fn test_get_gliders_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No gliders
        let a = get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(a.len(), 0);

        // Create some gliders
        diesel::insert_into(gliders::table)
            .values(vec![
                NewGlider {
                    user_id: ctx.testuser1.user.id,
                    model: "Epsilon 8 23".into(),
                    manufacturer: "Advance".into(),
                    ..Default::default()
                },
                NewGlider {
                    user_id: ctx.testuser1.user.id,
                    model: "Green S".into(),
                    manufacturer: "Team5".into(),
                    ..Default::default()
                },
                NewGlider {
                    user_id: ctx.testuser2.user.id,
                    model: "Pi 2".into(),
                    manufacturer: "Advance".into(),
                    ..Default::default()
                },
            ])
            .execute(&*ctx.force_get_conn())
            .expect("Could not create gliders");

        let a1 = get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser1.user);
        let a2 = get_gliders_for_user(&*ctx.force_get_conn(), &ctx.testuser2.user);
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
