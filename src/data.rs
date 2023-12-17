use std::{env, fmt};

use diesel::{
    dsl::{count, exists, max, select},
    prelude::*,
    result::{Error, QueryResult},
    sql_types::{BigInt, Bool, Double, Integer, Nullable, SmallInt, Text},
    {sql_function, sql_query, PgConnection},
};
use diesel_geography::{sql_types::Geography, types::GeogPoint};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, HarnessWithOutput, MigrationHarness};
use log::error;
use regex::Regex;
use rocket_sync_db_pools::database;
use serde::Serialize;

use crate::{
    models::{
        Flight, Glider, GliderWithStats, Igc, Location, LocationWithCount, LocationWithDistance, NewFlight,
        NewGlider, NewLocation, User,
    },
    schema::{flights, gliders, igcs, locations, users},
};

sql_function! {
    /// The pgcrypto "crypt" function.
    fn crypt(pw: Text, salt: Text) -> Text;
}

sql_function! {
    /// The pgcrypto "gen_salt" function.
    fn gen_salt(type_: Text, iter_count: Integer) -> Text;
}

const PW_SALT_ITERATIONS: i32 = 10;
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Database connection state object.
#[database("flugbuech")]
pub struct Database(diesel::PgConnection);

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Run migrations on the database indicated with `DATABASE_URL`.
pub fn run_migrations() -> Result<(), String> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    match PgConnection::establish(&database_url) {
        Ok(mut conn) => {
            HarnessWithOutput::write_to_stdout(&mut conn)
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| format!("Could not run migrations: {}", e))?;
            Ok(())
        }
        Err(e) => Err(format!("Could not connect to database: {}", e)),
    }
}

/// Return the user model with the specified user id.
pub fn get_user(conn: &mut PgConnection, id: i32) -> Option<User> {
    users::table
        .find(id)
        .first(conn)
        .map_err(|e| {
            error!("Could not query user: {}", e);
            e
        })
        .ok()
}

#[derive(Debug, PartialEq)]
pub enum RegistrationError {
    /// E-Mail is invalid or already registered
    InvalidEmail,
    /// Username is invalid or already taken
    NonUniqueUsername,
    /// The password confirmation does not match
    PasswordConfirmation,
    /// Password is too short
    InvalidPasswordFormat,
}

impl std::error::Error for RegistrationError {}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegistrationError::InvalidEmail => write!(f, "Invalid email format or email is not unique"),
            RegistrationError::InvalidPasswordFormat => {
                write!(f, "Password is too short or does not match format")
            }
            RegistrationError::NonUniqueUsername => write!(f, "Username is not unique"),
            RegistrationError::PasswordConfirmation => {
                write!(f, "Password and password confirmation do not match")
            }
        }
    }
}

/// Validate registration params and check for unique attributes { email, username }
pub fn validate_registration(
    conn: &mut PgConnection,
    email: &str,
    username: &str,
    password: &str,
    password_confirmation: &str,
) -> Result<(), RegistrationError> {
    validate_email(email)?;
    validate_password_confirmation_match(password, password_confirmation)?;
    validate_password_format(password)?;
    validate_unique_registration_fields(conn, email, username)?;
    Ok(())
}

fn validate_unique_registration_fields(
    conn: &mut PgConnection,
    email: &str,
    username: &str,
) -> Result<(), RegistrationError> {
    let user: Option<User> = users::table
        .filter(users::username.eq(username))
        .or_filter(users::email.eq(email))
        .first(conn)
        .ok();
    match user {
        Some(user) if user.username == username => Err(RegistrationError::NonUniqueUsername),
        Some(user) if user.email == email => Err(RegistrationError::InvalidEmail),
        Some(_) => unreachable!("Returned user must match with either email or username"),
        None => Ok(()),
    }
}

/// Validates that password and confirmation match
fn validate_password_confirmation_match(
    password: &str,
    password_confirmation: &str,
) -> Result<(), RegistrationError> {
    if password == password_confirmation {
        Ok(())
    } else {
        Err(RegistrationError::PasswordConfirmation)
    }
}

/// Validates the email based on an email regex which should cover most cases
fn validate_email(email: &str) -> Result<(), RegistrationError> {
    Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")
        .unwrap()
        .is_match(email)
        .then_some(())
        .ok_or(RegistrationError::InvalidEmail)
}

// TODO: extend with proper format check
/// Validates the password -> { length }
fn validate_password_format(password: &str) -> Result<(), RegistrationError> {
    if password.len() >= MIN_PASSWORD_LENGTH {
        Ok(())
    } else {
        Err(RegistrationError::InvalidPasswordFormat)
    }
}

/// Validate username / password combination. Return the corresponding user model if it is valid.
pub fn validate_login(conn: &mut PgConnection, username: &str, password: &str) -> Option<User> {
    users::table
        .filter(users::username.eq(username))
        .filter(users::password.eq(crypt(password, users::password)))
        .first(conn)
        .ok()
}

pub fn get_user_count(conn: &mut PgConnection) -> i64 {
    users::table
        .select(count(users::id))
        .first(conn)
        .expect("Error loading user count")
}

/// Create a user in the database. The password will be hashed.
pub fn create_user(
    conn: &mut PgConnection,
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

/// Update a user password, return the updated user model.
pub fn update_password(conn: &mut PgConnection, user: &User, password: impl Into<String>) -> User {
    diesel::update(user)
        .set(users::password.eq(crypt(password.into(), gen_salt("bf", PW_SALT_ITERATIONS))))
        .get_result(conn)
        .expect("Could not update user password")
}

pub fn get_glider_count(conn: &mut PgConnection) -> i64 {
    gliders::table
        .select(count(gliders::id))
        .first(conn)
        .expect("Error loading glider count")
}

pub fn get_gliders_for_user(conn: &mut PgConnection, user: &User) -> Vec<Glider> {
    Glider::belonging_to(user)
        .load(conn)
        .expect("Error loading gliders")
}

pub fn get_gliders_with_stats_for_user(conn: &mut PgConnection, user: &User) -> Vec<GliderWithStats> {
    sql_query(
        "SELECT g.*,
                count(f.id) as flights,
                coalesce(extract(epoch from sum(f.landing_time - f.launch_time))::bigint, 0) as seconds,
                every(f.launch_time IS NOT NULL AND f.landing_time IS NOT NULL) OR count(f.id) = 0 as seconds_complete
           FROM gliders g
                LEFT JOIN flights f ON g.id = f.glider_id
          WHERE g.user_id = $1
          GROUP BY g.id
          ORDER BY g.id DESC",
    )
    .bind::<Integer, _>(user.id)
    .load::<GliderWithStats>(conn)
    .expect("Error loading gliders with stats")
}

pub fn get_glider_with_id(conn: &mut PgConnection, id: i32) -> Option<Glider> {
    gliders::table
        .find(id)
        .first(conn)
        .optional()
        .expect("Error loading glider by id")
}

/// Create a new flight.
pub fn create_flight(conn: &mut PgConnection, flight: &NewFlight, igc: Option<Vec<u8>>) -> Flight {
    conn.transaction::<Flight, Error, _>(|conn| {
        // Create flight
        let flight: Flight = diesel::insert_into(flights::table)
            .values(flight)
            .get_result(conn)?;

        // Store IGC data
        if let Some(data) = igc {
            let val = Igc {
                flight_id: flight.id,
                data,
            };
            diesel::insert_into(igcs::table).values(val).execute(conn)?;
        }

        Ok(flight)
    })
    .expect("Transaction for create_flight failed")
}

/// Save an updated flight in the database.
pub fn update_flight(conn: &mut PgConnection, flight: &Flight) {
    diesel::update(flight)
        .set(flight)
        .execute(conn)
        .expect("Could not update flight");
}

/// Delete a flight.
pub fn delete_flight(conn: &mut PgConnection, flight: Flight) -> QueryResult<()> {
    let delete_count = diesel::delete(&flight).execute(conn)?;
    assert_eq!(delete_count, 1); // Sanity check
    Ok(())
}

pub fn get_flight_count(conn: &mut PgConnection) -> i64 {
    flights::table
        .select(count(flights::id))
        .first(conn)
        .expect("Error loading flight count")
}

/// Retrieve all flights of a specific user.
pub fn get_flights_for_user(conn: &mut PgConnection, user: &User) -> Vec<Flight> {
    Flight::belonging_to(user)
        .order((flights::number.desc(), flights::launch_time.desc()))
        .load(conn)
        .expect("Error loading flights")
}

/// Retrieve the highest flight number for a specific user.
pub fn get_max_flight_number_for_user(conn: &mut PgConnection, user: &User) -> Option<i32> {
    Flight::belonging_to(user)
        .select(max(flights::number))
        .first(conn)
        .expect("Error loading flight count")
}

/// Retrieve flight with the specified ID.
pub fn get_flight_with_id(conn: &mut PgConnection, id: i32) -> Option<Flight> {
    flights::table
        .find(id)
        .first(conn)
        .optional()
        .expect("Error loading flight by id")
}

/// Save an updated IGC entry in the database.
pub fn update_igc(conn: &mut PgConnection, flight: &Flight, data: &[u8]) {
    diesel::update(igcs::table.filter(igcs::flight_id.eq(flight.id)))
        .set(igcs::data.eq(data))
        .execute(conn)
        .expect("Could not update IGC entry");
}

/// Retrieve IGC data for the specified flight.
pub fn get_igc_for_flight(conn: &mut PgConnection, flight: &Flight) -> Option<Igc> {
    igcs::table
        .filter(igcs::flight_id.eq(flight.id))
        .first(conn)
        .optional()
        .expect("Error loading IGC by flight id")
}

/// Return all flight IDs of flights belonging to the specified user, where IGC
/// data is available.
pub fn get_flight_ids_with_igc_for_user(conn: &mut PgConnection, user: &User) -> Vec<i32> {
    Flight::belonging_to(user)
        .inner_join(igcs::table)
        .select(flights::id)
        .load(conn)
        .expect("Error loading flight ids with IGC for user")
}

/// Return whether or not the specified flight has associated IGC data.
pub fn flight_has_igc(conn: &mut PgConnection, flight: &Flight) -> bool {
    select(exists(igcs::table.filter(igcs::flight_id.eq(flight.id))))
        .get_result(conn)
        .expect("Error in flight_has_igc")
}

/// Retrieve all locations with the specified IDs.
pub fn get_locations_with_ids(conn: &mut PgConnection, ids: &[i32]) -> Vec<Location> {
    locations::table
        .filter(locations::id.eq_any(ids))
        .load(conn)
        .expect("Error loading locations")
}

/// Retrieve all locations for the specified user.
pub fn get_locations_for_user(conn: &mut PgConnection, user: &User) -> Vec<Location> {
    Location::belonging_to(user)
        .order(locations::name)
        .load(conn)
        .expect("Error loading locations")
}

/// Retrieve all locations for the specified user, including the number of
/// associated flights (with either launch and/or landing at that location).
pub fn get_all_locations_with_stats_for_user(conn: &mut PgConnection, user: &User) -> Vec<LocationWithCount> {
    sql_query(
        "SELECT l.*, count(f.id) as count
           FROM locations l
                LEFT JOIN flights f ON f.launch_at = l.id OR f.landing_at = l.id
          WHERE l.user_id = $1
          GROUP BY l.id
          ORDER BY l.name ASC",
    )
    .bind::<Integer, _>(user.id)
    .load(conn)
    .expect("Error loading locations with stats")
}

#[derive(Debug, PartialEq)]
pub enum LocationAggregateBy {
    Launches,
    Landings,
}

/// Retrieve all visited locations for the specified user, including either
/// launch or landing count.
///
/// Entries with a count of 0 will not be included.
pub fn get_visited_locations_with_stats_for_user(
    conn: &mut PgConnection,
    user: &User,
    aggregate_by: LocationAggregateBy,
    limit: i32,
) -> Vec<LocationWithCount> {
    sql_query(&format!(
        "SELECT l.*, count(f.*) as count
           FROM locations l
                LEFT JOIN flights f on f.{} = l.id
          WHERE f.user_id = $1 AND l.user_id = $1
          GROUP BY l.id
          ORDER BY count DESC
          LIMIT $2",
        match aggregate_by {
            LocationAggregateBy::Launches => "launch_at",
            LocationAggregateBy::Landings => "landing_at",
        },
    ))
    .bind::<Integer, _>(user.id)
    .bind::<Integer, _>(limit)
    .load(conn)
    .expect("Error loading locations")
}

/// Retrieve all locations for the specified user within a specified radius
/// from the specified coordinates.
pub fn get_locations_around_point(
    conn: &mut PgConnection,
    user: &User,
    lat: f64,
    lng: f64,
    max_distance_meters: f64,
) -> Vec<LocationWithDistance> {
    let point = GeogPoint {
        x: lng,
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
pub fn get_location_by_id(conn: &mut PgConnection, id: i32) -> Option<Location> {
    locations::table
        .find(id)
        .first(conn)
        .optional()
        .expect("Error loading location by id")
}

/// Retrieve location with the specified ID and the number of associated flights.
pub fn get_location_with_flight_count_by_id(conn: &mut PgConnection, id: i32) -> Option<LocationWithCount> {
    sql_query(
        "SELECT l.*, count(f.id) as count
           FROM locations l
                LEFT JOIN flights f ON f.launch_at = l.id OR f.landing_at = l.id
          WHERE l.id = $1
          GROUP BY l.id",
    )
    .bind::<Integer, _>(id)
    .get_result(conn)
    .optional()
    .expect("Error loading location by id with flight count")
}

/// Create a new location.
pub fn create_location(conn: &mut PgConnection, location: NewLocation) -> Location {
    diesel::insert_into(locations::table)
        .values(location)
        .get_result(conn)
        .expect("Could not create location")
}

/// Save an updated location in the database.
pub fn update_location(conn: &mut PgConnection, location: &Location) {
    diesel::update(location)
        .set(location)
        .execute(conn)
        .expect("Could not update location");
}

/// Delete a location by ID.
pub fn delete_location_by_id(conn: &mut PgConnection, id: i32) -> QueryResult<()> {
    let delete_count = diesel::delete(locations::table.filter(locations::id.eq(&id))).execute(conn)?;
    assert_eq!(delete_count, 1); // Sanity check
    Ok(())
}

/// Create a new glider.
pub fn create_glider(conn: &mut PgConnection, glider: NewGlider) -> QueryResult<Glider> {
    diesel::insert_into(gliders::table)
        .values(glider)
        .get_result(conn)
}

/// Save an updated glider in the database.
pub fn update_glider(conn: &mut PgConnection, glider: &Glider) {
    diesel::update(glider)
        .set(glider)
        .execute(conn)
        .expect("Could not update glider");
}

/// Update the "last glider" of a user.
pub fn update_user_last_glider(conn: &mut PgConnection, user: &User, glider_id: i32) {
    diesel::update(user)
        .set(users::last_glider_id.eq(glider_id))
        .execute(conn)
        .expect("Could not set user last glider id");
}

#[derive(Debug, QueryableByName)]
pub struct FlightCount {
    #[sql_type = "SmallInt"]
    pub year: i16,
    #[sql_type = "BigInt"]
    pub count: i64,
}

/// Get flight count per year for the specified user.
pub fn get_flight_count_per_year_for_user(conn: &mut PgConnection, user: &User) -> Vec<FlightCount> {
    sql_query(
        "SELECT date_part('year', launch_time)::smallint as year,
                count(*) as count
           FROM flights
          WHERE user_id = $1
            AND launch_time IS NOT NULL
          GROUP BY year
          ORDER BY year DESC",
    )
    .bind::<Integer, _>(user.id)
    .load::<FlightCount>(conn)
    .expect("Error loading flight count stats")
}

/// Get hike&fly count per year for the specified user.
pub fn get_hikeandfly_count_per_year_for_user(conn: &mut PgConnection, user: &User) -> Vec<FlightCount> {
    sql_query(
        "SELECT date_part('year', launch_time)::smallint as year,
                count(*) as count
           FROM flights
          WHERE user_id = $1
            AND launch_time IS NOT NULL
            AND hikeandfly = true
          GROUP BY year
          ORDER BY year DESC",
    )
    .bind::<Integer, _>(user.id)
    .load::<FlightCount>(conn)
    .expect("Error loading hike&fly count stats")
}

#[derive(Debug, QueryableByName)]
pub struct FlightTime {
    #[sql_type = "SmallInt"]
    pub year: i16,
    #[sql_type = "BigInt"]
    pub seconds: i64,
}

/// Get flight hours per year for the specified user.
pub fn get_flight_time_per_year_for_user(conn: &mut PgConnection, user: &User) -> Vec<FlightTime> {
    sql_query(
        "SELECT date_part('year', launch_time)::smallint as year,
                extract(epoch from sum(landing_time - launch_time))::bigint as seconds
           FROM flights
          WHERE user_id = $1
            AND launch_time IS NOT NULL
          GROUP BY year
          ORDER BY year DESC",
    )
    .bind::<Integer, _>(user.id)
    .load::<FlightTime>(conn)
    .expect("Error loading flight time stats")
}

pub fn get_flight_count_without_launch_time(conn: &mut PgConnection, user: &User) -> i64 {
    Flight::belonging_to(user)
        .filter(flights::launch_time.is_null())
        .select(count(flights::id))
        .first::<i64>(conn)
        .expect("Error loading flight count without launch time")
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct FlightDistance {
    #[sql_type = "SmallInt"]
    pub year: i16,
    #[sql_type = "Nullable<Integer>"]
    pub track: Option<i32>,
    #[sql_type = "Bool"]
    pub track_incomplete: bool,
    #[sql_type = "Nullable<Integer>"]
    pub scored: Option<i32>,
    #[sql_type = "Bool"]
    pub scored_incomplete: bool,
}

/// Get flight distance per year for the specified user.
pub fn get_flight_distance_per_year_for_user(conn: &mut PgConnection, user: &User) -> Vec<FlightDistance> {
    sql_query(
        "SELECT date_part('year', launch_time)::smallint as year,
                sum(track_distance)::int as track,
                count(*) - count(track_distance) > 0 as track_incomplete,
                sum(xcontest_distance)::int as scored,
                count(*) - count(xcontest_distance) > 0 as scored_incomplete
           FROM flights
          WHERE user_id = $1
            AND launch_time IS NOT NULL
          GROUP BY year
          ORDER BY year DESC",
    )
    .bind::<Integer, _>(user.id)
    .load::<FlightDistance>(conn)
    .expect("Error loading flight distance stats")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{models::NewGlider, test_utils};

    use super::*;

    #[rstest]
    #[case("email.peter@gmail.com")]
    #[case("email@gmail.com")]
    #[case("em123@gmail.com")]
    #[case("em+@gmail.com")]
    fn test_valid_email(#[case] email: &str) {
        assert_eq!(validate_email(email), Ok(()));
    }

    #[rstest]
    #[case("@gmail.com", RegistrationError::InvalidEmail)]
    #[case("petergmail.com", RegistrationError::InvalidEmail)]
    fn test_invalid_email(#[case] email: &str, #[case] error: RegistrationError) {
        assert_eq!(validate_email(email), Err(error));
    }

    #[rstest]
    #[case("email@email.ch", "user", Ok(()))]
    #[case("example@example.com", "testuser1", Err(RegistrationError::NonUniqueUsername))]
    #[case("user1@example.com", "username", Err(RegistrationError::InvalidEmail))]
    fn test_duplicate_username(
        #[case] email: &str,
        #[case] username: &str,
        #[case] result: Result<(), RegistrationError>,
    ) {
        let ctx = test_utils::DbTestContext::new();
        assert_eq!(
            validate_unique_registration_fields(&mut ctx.force_get_conn(), email, username),
            result
        );
    }

    #[test]
    fn test_matching_password_confirmation() {
        assert_eq!(
            validate_password_confirmation_match("password", "password"),
            Ok(())
        );
        assert_eq!(
            validate_password_confirmation_match("password", "passwor"),
            Err(RegistrationError::PasswordConfirmation)
        );
    }

    #[rstest]
    #[case("password123")]
    #[case("password")]
    fn test_valid_password_format(#[case] password: &str) {
        assert_eq!(validate_password_format(password), Ok(()));
    }

    #[rstest]
    #[case("passwor")]
    #[case("")]
    fn test_invalid_password_format(#[case] password: &str) {
        assert_eq!(
            validate_password_format(password),
            Err(RegistrationError::InvalidPasswordFormat)
        );
    }

    #[test]
    fn test_get_max_flight_number_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No flights
        let n = get_max_flight_number_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(n, None);

        // Single flight, no number
        diesel::insert_into(flights::table)
            .values(NewFlight {
                number: None,
                user_id: ctx.testuser1.user.id,
                ..Default::default()
            })
            .execute(&mut *ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_max_flight_number_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
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
            .execute(&mut *ctx.force_get_conn())
            .expect("Could not create flight");
        let n = get_max_flight_number_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(n, Some(7));

        // The user id is properly taken into account
        let n = get_max_flight_number_for_user(&mut ctx.force_get_conn(), &ctx.testuser2.user);
        assert_eq!(n, None);
    }

    #[test]
    fn test_get_gliders_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No gliders
        let a = get_gliders_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
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
            .execute(&mut *ctx.force_get_conn())
            .expect("Could not create gliders");

        let a1 = get_gliders_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        let a2 = get_gliders_for_user(&mut ctx.force_get_conn(), &ctx.testuser2.user);
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
        let user = validate_login(&mut ctx.force_get_conn(), "foobar", "bazbong");
        assert!(user.is_none());
    }

    #[test]
    fn validate_login_invalid_bad_password() {
        let ctx = test_utils::DbTestContext::new();
        // Wrong password, this must fail
        let user = validate_login(&mut ctx.force_get_conn(), &ctx.testuser1.user.username, "bazbong");
        assert!(user.is_none());
    }

    #[test]
    fn validate_login_invalid_correct_password() {
        let ctx = test_utils::DbTestContext::new();
        // Correct password, this should succeed
        let user = validate_login(
            &mut ctx.force_get_conn(),
            &ctx.testuser1.user.username,
            &ctx.testuser1.password,
        );
        assert!(user.is_some());
        assert_eq!(user.unwrap().id, ctx.testuser1.user.id);
    }

    #[test]
    fn update_login_password() {
        let ctx = test_utils::DbTestContext::new();

        let oldpass = ctx.testuser1.password.clone();
        let newpass = "aabbccdd".to_string();
        assert_ne!(oldpass, newpass);

        // Correct password, this should succeed
        let user = validate_login(&mut ctx.force_get_conn(), &ctx.testuser1.user.username, &oldpass);
        assert!(user.is_some());

        // Change password
        update_password(&mut ctx.force_get_conn(), &user.unwrap(), &newpass);

        // Old password should not work anymore
        let user2 = validate_login(&mut ctx.force_get_conn(), &ctx.testuser1.user.username, &oldpass);
        assert!(user2.is_none());

        // New password should work
        let user3 = validate_login(&mut ctx.force_get_conn(), &ctx.testuser1.user.username, &newpass);
        assert!(user3.is_some());
    }

    #[test]
    fn test_get_visited_locations_with_stats_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No locations
        let l = get_visited_locations_with_stats_for_user(
            &mut ctx.force_get_conn(),
            &ctx.testuser1.user,
            LocationAggregateBy::Launches,
            99,
        );
        assert_eq!(l.len(), 0);

        // Locations, but no associated flights
        let locations = diesel::insert_into(locations::table)
            .values(vec![
                NewLocation {
                    name: "Selun".into(),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewLocation {
                    name: "Etzel".into(),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewLocation {
                    name: "Altendorf".into(),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewLocation {
                    name: "Hummel".into(),
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewLocation {
                    name: "Stöcklichrüz".into(),
                    user_id: ctx.testuser2.user.id,
                    ..Default::default()
                },
                NewLocation {
                    name: "Pfäffikon".into(),
                    user_id: ctx.testuser2.user.id,
                    ..Default::default()
                },
            ])
            .get_results::<Location>(&mut *ctx.force_get_conn())
            .expect("Could not create flight");
        let l = get_visited_locations_with_stats_for_user(
            &mut ctx.force_get_conn(),
            &ctx.testuser1.user,
            LocationAggregateBy::Launches,
            99,
        );
        assert_eq!(l.len(), 0);

        // Add some flights
        diesel::insert_into(flights::table)
            .values(vec![
                NewFlight {
                    // Selun - Unknown
                    user_id: ctx.testuser1.user.id,
                    launch_at: Some(locations[0].id),
                    landing_at: None,
                    ..Default::default()
                },
                NewFlight {
                    // Selun - Altendorf
                    user_id: ctx.testuser1.user.id,
                    launch_at: Some(locations[0].id),
                    landing_at: Some(locations[2].id),
                    ..Default::default()
                },
                NewFlight {
                    // Selun - Altendorf
                    user_id: ctx.testuser1.user.id,
                    launch_at: Some(locations[0].id),
                    landing_at: Some(locations[2].id),
                    ..Default::default()
                },
                NewFlight {
                    // Etzel - Altendorf
                    user_id: ctx.testuser1.user.id,
                    launch_at: Some(locations[1].id),
                    landing_at: Some(locations[2].id),
                    ..Default::default()
                },
                NewFlight {
                    // Etzel - Etzel (toplanding)
                    user_id: ctx.testuser1.user.id,
                    launch_at: Some(locations[1].id),
                    landing_at: Some(locations[1].id),
                    ..Default::default()
                },
                NewFlight {
                    // Stöcklichrüz - Pfäffikon (other user)
                    user_id: ctx.testuser2.user.id,
                    launch_at: Some(locations[4].id),
                    landing_at: Some(locations[5].id),
                    ..Default::default()
                },
            ])
            .execute(&mut *ctx.force_get_conn())
            .expect("Could not create flight");
        let l_launches = get_visited_locations_with_stats_for_user(
            &mut ctx.force_get_conn(),
            &ctx.testuser1.user,
            LocationAggregateBy::Launches,
            99,
        )
        .into_iter()
        .map(|l| (l.name, l.count))
        .collect::<Vec<_>>();
        assert_eq!(l_launches, vec![("Selun".into(), 3), ("Etzel".into(), 2)]);
        let l_landings = get_visited_locations_with_stats_for_user(
            &mut ctx.force_get_conn(),
            &ctx.testuser1.user,
            LocationAggregateBy::Landings,
            99,
        )
        .into_iter()
        .map(|l| (l.name, l.count))
        .collect::<Vec<_>>();
        assert_eq!(l_landings, vec![("Altendorf".into(), 3), ("Etzel".into(), 1)]);
    }

    #[test]
    fn test_get_flight_ids_with_igc_for_user() {
        let ctx = test_utils::DbTestContext::new();

        // No flights with IGC
        let ids = get_flight_ids_with_igc_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(ids.len(), 0);

        // Add some flights
        let flights = diesel::insert_into(flights::table)
            .values(vec![
                NewFlight {
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    user_id: ctx.testuser1.user.id,
                    ..Default::default()
                },
                NewFlight {
                    user_id: ctx.testuser2.user.id,
                    ..Default::default()
                },
            ])
            .get_results::<Flight>(&mut *ctx.force_get_conn())
            .expect("Could not create flights");

        // Still no flights with IGC
        let ids = get_flight_ids_with_igc_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(ids.len(), 0);

        // Add some IGC files
        for i in &[0, 2, 3] {
            diesel::insert_into(igcs::table)
                .values(&(igcs::flight_id.eq(flights[*i].id), igcs::data.eq(vec![1, 2, 3])))
                .execute(&mut *ctx.force_get_conn())
                .expect("Could not create IGC entry");
        }

        // Two IGC files for user 1
        let result = get_flight_ids_with_igc_for_user(&mut ctx.force_get_conn(), &ctx.testuser1.user);
        assert_eq!(result, vec![flights[0].id, flights[2].id]);

        // One IGC file for user 2
        let result = get_flight_ids_with_igc_for_user(&mut ctx.force_get_conn(), &ctx.testuser2.user);
        assert_eq!(result, vec![flights[3].id]);
    }
}
