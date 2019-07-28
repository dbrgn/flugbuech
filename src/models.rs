use chrono::{DateTime, offset::Utc};
use diesel::{Associations, Identifiable, Queryable};
use serde::Serialize;

use crate::schema::{users, aircraft, locations, flights};

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug, Clone)]
#[table_name="users"]
pub struct User {
    pub id: i32,
    pub username: String,
    /// Password is automatically hashed on insert or update
    /// by a PostgreSQL trigger.
    pub password: String,
}

#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key="user_id")]
#[table_name="aircraft"]
pub struct Aircraft {
    pub id: i32,
    pub user_id: i32,
    pub model: String,
    pub manufacturer: String,
}

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug, Clone)]
#[table_name="locations"]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub country: String,
    pub elevation: i32,
}

#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug, Clone)]
#[belongs_to(User, foreign_key="user_id")]
#[belongs_to(Aircraft, foreign_key="aircraft_id")]
#[belongs_to(Location, foreign_key="launch_at")]
#[belongs_to(Location, foreign_key="landing_at")]
#[table_name="flights"]
pub struct Flight {
    /// Primary key
    pub id: i32,
    /// The user-defined flight number
    pub number: i32,
    /// The pilot
    pub user_id: i32,
    /// The aircraft
    pub aircraft_id: Option<i32>,
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
}
