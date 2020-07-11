use chrono::{DateTime, NaiveDate, Utc};
use diesel::sql_types::{BigInt, Double, Integer, Text};
use diesel::{Associations, Identifiable, Queryable};
use diesel_geography::sql_types::Geography;
use diesel_geography::types::GeogPoint;
use serde::Serialize;

use crate::schema::{flights, gliders, locations, users};

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    /// Password is automatically hashed on insert or update
    /// by a PostgreSQL trigger.
    pub password: String,
    /// Last used glider
    pub last_glider_id: Option<i32>,
    /// E-mail address
    pub email: String,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "gliders"]
pub struct Glider {
    pub id: i32,
    pub user_id: i32,
    /// The model name, e.g. "Epsilon 8"
    pub model: String,
    /// The manufacturer name, e.g. "Advance"
    pub manufacturer: String,
    /// When was the glider acquired?
    pub since: Option<NaiveDate>,
    /// When was the glider sold / given away / thrown away?
    pub until: Option<NaiveDate>,
    /// Where did you get the glider from? (e.g. a shop, or a website)
    pub source: Option<String>,
    /// How much did the glider cost, in your currency?
    pub cost: Option<i32>,
    /// Add arbitrary comments about this glider
    pub comment: Option<String>,
}

#[derive(Insertable, Default)]
#[table_name = "gliders"]
pub struct NewGlider {
    pub user_id: i32,
    pub model: String,
    pub manufacturer: String,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub source: Option<String>,
    pub cost: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "locations"]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub country: String,
    pub elevation: i32,
    pub user_id: i32,
    pub geog: Option<GeogPoint>,
}

#[derive(Insertable, Default)]
#[table_name = "locations"]
pub struct NewLocation {
    pub name: String,
    pub country: String,
    pub elevation: i32,
    pub user_id: i32,
    pub geog: Option<GeogPoint>,
}

#[derive(QueryableByName, PartialEq, Debug, Clone)]
pub struct LocationWithDistance {
    #[sql_type = "Integer"]
    pub id: i32,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Text"]
    pub country: String,
    #[sql_type = "Integer"]
    pub elevation: i32,
    #[sql_type = "Integer"]
    pub user_id: i32,
    #[sql_type = "Geography"]
    pub geog: GeogPoint,
    #[sql_type = "Double"]
    pub distance: f64,
}

/// Locations with a count (e.g. landing count).
#[derive(QueryableByName, Serialize, PartialEq, Debug, Clone)]
pub struct LocationWithCount {
    #[sql_type = "Integer"]
    pub id: i32,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Text"]
    pub country: String,
    #[sql_type = "Integer"]
    pub elevation: i32,
    #[sql_type = "Integer"]
    pub user_id: i32,
    #[sql_type = "BigInt"]
    pub count: i64,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(Glider, foreign_key = "glider_id")]
#[table_name = "flights"]
pub struct Flight {
    /// Primary key
    pub id: i32,
    /// The user-defined flight number
    pub number: Option<i32>,
    /// The pilot
    pub user_id: i32,
    /// The glider
    pub glider_id: Option<i32>,
    /// Launch location
    pub launch_at: Option<i32>,
    /// Landing location
    pub landing_at: Option<i32>,
    /// Time of launch
    pub launch_time: Option<DateTime<Utc>>,
    /// Time of landing
    pub landing_time: Option<DateTime<Utc>>,
    /// GPS track length
    pub track_distance: Option<f32>,
    /// XContest tracktype (free_flight, flat_triangle or fai_triangle)
    pub xcontest_tracktype: Option<String>,
    /// XContest distance
    pub xcontest_distance: Option<f32>,
    /// XContest URL
    pub xcontest_url: Option<String>,
    /// Comment your flight
    pub comment: Option<String>,
    /// Link to a video of your flight
    pub video_url: Option<String>,
    /// IGC file contents
    pub igc: Option<Vec<u8>>,
    /// When the flight was entered into Flugbuech
    pub created_at: DateTime<Utc>,
    /// Whether you hiked up to launch
    pub hikeandfly: bool,
}

#[derive(Insertable, Default)]
#[table_name = "flights"]
pub struct NewFlight {
    /// The user-defined flight number
    pub number: Option<i32>,
    /// The pilot
    pub user_id: i32,
    /// The glider
    pub glider_id: Option<i32>,
    /// Launch location
    pub launch_at: Option<i32>,
    /// Landing location
    pub landing_at: Option<i32>,
    /// Time of launch
    pub launch_time: Option<DateTime<Utc>>,
    /// Time of landing
    pub landing_time: Option<DateTime<Utc>>,
    /// GPS track length
    pub track_distance: Option<f32>,
    /// XContest tracktype (free_flight, flat_triangle or fai_triangle)
    pub xcontest_tracktype: Option<String>,
    /// XContest distance
    pub xcontest_distance: Option<f32>,
    /// XContest URL
    pub xcontest_url: Option<String>,
    /// Comment your flight
    pub comment: Option<String>,
    /// Link to a video of your flight
    pub video_url: Option<String>,
    /// IGC file contents
    pub igc: Option<Vec<u8>>,
    /// Whether you hiked up to launch
    pub hikeandfly: bool,
}
