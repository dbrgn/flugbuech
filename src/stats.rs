//! Stats views.

use std::collections::BTreeMap;

use rocket::{get, response::Redirect};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{
    auth,
    data::{self, LocationOrderBy},
    models::{LocationWithCount, User},
};

// Contexts

#[derive(Default, Serialize)]
struct YearStats {
    flight_seconds: Option<u64>,
    distance_track: Option<i32>,
    distance_scored: Option<i32>,
}

#[derive(Serialize)]
struct StatsContext {
    user: User,
    launch_locations: Vec<LocationWithCount>,
    landing_locations: Vec<LocationWithCount>,
    yearly_stats: BTreeMap<u16, YearStats>,
    flight_time_total: u64,
    flight_distance_total: (i32, i32), // (track, scored)
}

// Views

#[get("/stats")]
pub(crate) fn stats(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all locations
    let launch_locations = data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Launches, 10);
    let landing_locations =
        data::get_locations_with_stats_for_user(&db, &user, LocationOrderBy::Landings, 10);

    // Yearly stats map
    let mut yearly_stats: BTreeMap<u16, YearStats> = BTreeMap::new();

    // Get hours per year
    for (year, seconds) in data::get_flight_time_per_year_for_user(&db, &user) {
        yearly_stats.entry(year).or_default().flight_seconds = Some(seconds);
    }
    let flight_time_total = yearly_stats.values().filter_map(|s| s.flight_seconds).sum();

    // Get km per year
    for distance in data::get_flight_distance_per_year_for_user(&db, &user) {
        let mut stats = yearly_stats.entry(distance.year as u16).or_default();
        stats.distance_track = distance.track;
        stats.distance_scored = distance.scored;
    }
    let flight_distance_total = (
        yearly_stats.values().filter_map(|s| s.distance_track).sum(),
        yearly_stats.values().filter_map(|s| s.distance_scored).sum(),
    );

    // Render template
    let context = StatsContext {
        user,
        launch_locations,
        landing_locations,
        yearly_stats,
        flight_time_total,
        flight_distance_total,
    };
    Template::render("stats", &context)
}

#[get("/stats", rank = 2)]
pub(crate) fn stats_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
