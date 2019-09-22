//! Flight views.

use std::collections::HashMap;

use rocket::get;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{auth, data};
use crate::models::{Location, Flight, Aircraft, User};

#[derive(Serialize)]
struct FlightWithDetails<'a> {
    flight: Flight,
    aircraft: Option<&'a Aircraft>,
    launch_at: Option<&'a Location>,
    landing_at: Option<&'a Location>,
    duration_seconds: Option<u64>,
}

#[derive(Serialize)]
struct FlightsContext<'a> {
    user: User,
    flights: Vec<FlightWithDetails<'a>>,
}

#[get("/flights")]
pub fn flights(db: data::Database, user: auth::AuthUser) -> Template {
    let user = user.into_inner();

    // Get all flights
    let flights = data::get_flights_for_user(&db, &user);

    // Get all aircraft for user
    let aircraft_map = data::get_aircraft_for_user(&db, &user)
        .into_iter()
        .map(|aircraft| (aircraft.id, aircraft))
        .collect::<HashMap<i32, Aircraft>>();

    // Get all locations used
    let mut location_ids = flights
        .iter()
        .flat_map(|flight| vec![flight.launch_at, flight.landing_at])
        .filter_map(|opt| opt)
        .collect::<Vec<_>>();
    location_ids.sort();
    location_ids.dedup();
    let location_map = data::get_locations_with_ids(&db, &location_ids)
        .into_iter()
        .map(|location| (location.id, location))
        .collect::<HashMap<i32, Location>>();

    // Add details to flights
    let flights_with_details = flights
        .into_iter()
        .map(|flight| {
            // Look up aircraft
            let aircraft = flight.aircraft_id.and_then(|id| aircraft_map.get(&id));

            // Look up launch and landing
            let launch_at = flight.launch_at.and_then(|id| location_map.get(&id));
            let landing_at = flight.landing_at.and_then(|id| location_map.get(&id));

            // Calculate duration
            let duration_seconds = match (flight.launch_time, flight.landing_time) {
                (Some(launch), Some(landing)) => {
                    let duration = (landing - launch).num_seconds();
                    if duration < 0 {
                        None
                    } else {
                        Some(duration as u64)
                    }
                },
                _ => None,
            };
            FlightWithDetails {
                flight,
                aircraft,
                launch_at,
                landing_at,
                duration_seconds,
            }
        })
        .collect::<Vec<_>>();

    // Render template
    let context = FlightsContext {
        user,
        flights: flights_with_details,
    };
    Template::render("flights", &context)
}

#[get("/flights", rank = 2)]
pub fn flights_nologin() -> Redirect {
    Redirect::to("/auth/login")
}
