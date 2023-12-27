//! Stats views.

use std::collections::BTreeMap;

use rocket::{
    get,
    response::Redirect,
    serde::{json::Json, Serialize},
};
use rocket_dyn_templates::Template;

use crate::{
    auth,
    data::{self, LocationAggregateBy},
    models::{LocationWithCount, User},
};

// Contexts

#[derive(Default, Serialize)]
struct YearStats {
    flight_count: Option<u32>,
    hikeandfly_count: Option<u32>,
    flight_seconds: Option<u64>,
    distance_track: Option<i32>,
    distance_track_incomplete: bool,
    distance_scored: Option<i32>,
    distance_scored_incomplete: bool,
}

#[derive(Serialize)]
struct StatsContext {
    user: User,
    launch_locations: Vec<LocationWithCount>,
    landing_locations: Vec<LocationWithCount>,
    yearly_stats: BTreeMap<String, YearStats>,
    flight_count_total: u32,
    hikeandfly_count_total: u32,
    flight_time_total: u64,
    flight_distance_total: (i32, i32), // (track, scored)
    flights_without_launch_time: u64,
}

// Views

#[get("/stats")]
pub async fn stats(database: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    let context = database
        .run(move |db| {
            // Get all locations
            let launch_locations =
                data::get_visited_locations_with_stats_for_user(db, &user, LocationAggregateBy::Launches, 10);
            let landing_locations =
                data::get_visited_locations_with_stats_for_user(db, &user, LocationAggregateBy::Landings, 10);

            // Yearly stats map
            let mut yearly_stats: BTreeMap<String, YearStats> = BTreeMap::new();

            // Determine data completeness
            let flights_without_launch_time = data::get_flight_count_without_launch_time(db, &user) as u64;

            // Get flight count per year
            for count in data::get_flight_count_per_year_for_user(db, &user) {
                yearly_stats
                    .entry(count.year.to_string())
                    .or_default()
                    .flight_count = Some(count.count as u32);
            }
            let flight_count_total = yearly_stats.values().filter_map(|s| s.flight_count).sum();

            // Get hike&fly count per year
            for count in data::get_hikeandfly_count_per_year_for_user(db, &user) {
                yearly_stats
                    .entry(count.year.to_string())
                    .or_default()
                    .hikeandfly_count = Some(count.count as u32);
            }
            let hikeandfly_count_total = yearly_stats.values().filter_map(|s| s.hikeandfly_count).sum();

            // Get hours per year
            for time in data::get_flight_time_per_year_for_user(db, &user) {
                yearly_stats
                    .entry(time.year.to_string())
                    .or_default()
                    .flight_seconds = Some(time.seconds as u64);
            }
            let flight_time_total = yearly_stats.values().filter_map(|s| s.flight_seconds).sum();

            // Get km per year
            for distance in data::get_flight_distance_per_year_for_user(db, &user) {
                let stats = yearly_stats.entry(distance.year.to_string()).or_default();
                stats.distance_track = distance.track;
                stats.distance_track_incomplete = distance.track_incomplete;
                stats.distance_scored = distance.scored;
                stats.distance_scored_incomplete = distance.scored_incomplete;
            }
            let flight_distance_total = (
                yearly_stats.values().filter_map(|s| s.distance_track).sum(),
                yearly_stats.values().filter_map(|s| s.distance_scored).sum(),
            );

            // Render template
            StatsContext {
                user,
                launch_locations,
                landing_locations,
                yearly_stats,
                flight_count_total,
                hikeandfly_count_total,
                flight_time_total,
                flight_distance_total,
                flights_without_launch_time,
            }
        })
        .await;
    Template::render("stats", &context)
}

#[get("/stats", rank = 2)]
pub fn stats_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

// API Endpoints

#[derive(Serialize)]
pub struct GlobalStats {
    user_count: i64,
    glider_count: i64,
    flight_count: i64,
}

#[get("/global-stats")]
pub async fn global_stats(database: data::Database) -> Json<GlobalStats> {
    let (user_count, glider_count, flight_count) = database
        .run(|db| {
            (
                data::get_user_count(db),
                data::get_glider_count(db),
                data::get_flight_count(db),
            )
        })
        .await;
    Json(GlobalStats {
        user_count,
        glider_count,
        flight_count,
    })
}
