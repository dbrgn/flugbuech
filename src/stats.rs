//! Stats views.

use std::{collections::BTreeMap, convert::TryInto};

use rocket::{
    get, routes,
    serde::{json::Json, Serialize},
    Route,
};

use crate::{
    auth,
    data::{self, LocationAggregateBy},
    locations::ApiLocation,
    responders::ApiError,
};

// API types

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiDistance {
    track: u32,
    scored: u32,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiYearStats {
    flight_count: u32,
    hikeandfly_count: u32,
    flight_seconds: u64,
    distance: ApiDistance,
    distance_track_incomplete: bool,
    distance_scored_incomplete: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStats {
    launch_locations: Vec<ApiLocation>,
    landing_locations: Vec<ApiLocation>,
    yearly_stats: BTreeMap<u16, ApiYearStats>,
    flight_count_total: u32,
    hikeandfly_count_total: u32,
    flight_time_total: u64,
    flight_distance_total: ApiDistance,
    flights_without_launch_time: u64,
}

#[derive(Serialize)]
pub struct ApiGlobalStats {
    user_count: i64,
    glider_count: i64,
    flight_count: i64,
}

// API endpoints

#[get("/stats")]
pub async fn stats(database: data::Database, user: auth::AuthUser) -> Json<ApiStats> {
    let user = user.into_inner();

    let stats = database
        .run(move |db| {
            // Get all locations
            let launch_locations: Vec<ApiLocation> =
                data::get_visited_locations_with_stats_for_user(db, &user, LocationAggregateBy::Launches, 10)
                    .into_iter()
                    .map(|location| location.into())
                    .collect();
            let landing_locations: Vec<ApiLocation> =
                data::get_visited_locations_with_stats_for_user(db, &user, LocationAggregateBy::Landings, 10)
                    .into_iter()
                    .map(|location| location.into())
                    .collect();

            // Yearly stats map
            let mut yearly_stats: BTreeMap<u16, ApiYearStats> = BTreeMap::new();

            // Determine data completeness
            let flights_without_launch_time = data::get_flight_count_without_launch_time(db, &user) as u64;

            // Get flight count per year
            for count in data::get_flight_count_per_year_for_user(db, &user) {
                yearly_stats.entry(count.year as u16).or_default().flight_count = count.count as u32;
            }
            let flight_count_total = yearly_stats.values().map(|s| s.flight_count).sum();

            // Get hike&fly count per year
            for count in data::get_hikeandfly_count_per_year_for_user(db, &user) {
                yearly_stats
                    .entry(count.year as u16)
                    .or_default()
                    .hikeandfly_count = count.count as u32;
            }
            let hikeandfly_count_total = yearly_stats.values().map(|s| s.hikeandfly_count).sum();

            // Get hours per year
            for time in data::get_flight_time_per_year_for_user(db, &user) {
                yearly_stats.entry(time.year as u16).or_default().flight_seconds = time.seconds as u64;
            }
            let flight_time_total = yearly_stats.values().map(|s| s.flight_seconds).sum();

            // Get km per year
            for distance in data::get_flight_distance_per_year_for_user(db, &user) {
                let stats = yearly_stats.entry(distance.year as u16).or_default();
                stats.distance = ApiDistance {
                    track: distance
                        .track
                        .map(|value| {
                            value
                                .try_into()
                                .expect("Yearly stats track distance overflows u32")
                        })
                        .unwrap_or(0),
                    scored: distance
                        .scored
                        .map(|value| {
                            value
                                .try_into()
                                .expect("Yearly stats scored distance overflows u32")
                        })
                        .unwrap_or(0),
                };
                stats.distance_track_incomplete = distance.track_incomplete;
                stats.distance_scored_incomplete = distance.scored_incomplete;
            }
            let flight_distance_total = ApiDistance {
                track: yearly_stats.values().map(|s| s.distance.track).sum(),
                scored: yearly_stats.values().map(|s| s.distance.scored).sum(),
            };

            // Render template
            ApiStats {
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

    Json(stats)
}

#[get("/stats", rank = 2)]
pub fn stats_nologin() -> ApiError {
    ApiError::MissingAuthentication
}

#[get("/global-stats")]
pub async fn global_stats(database: data::Database) -> Json<ApiGlobalStats> {
    let (user_count, glider_count, flight_count) = database
        .run(|db| {
            (
                data::get_user_count(db),
                data::get_glider_count(db),
                data::get_flight_count(db),
            )
        })
        .await;
    Json(ApiGlobalStats {
        user_count,
        glider_count,
        flight_count,
    })
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![stats, stats_nologin, global_stats]
}
