#![allow(clippy::needless_borrow)]

#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

mod auth;
mod cors;
mod data;
mod flights;
mod gliders;
mod import_csv;
mod locations;
mod models;
mod process_igc;
mod profile;
mod responders;
mod schema;
mod stats;
#[cfg(test)]
mod test_utils;
mod xcontest;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use rocket::{catch, catchers, get, request::Request, routes};
use serde::Deserialize;

// Limits
//
// Note: Other limits are configured in Rocket.toml!
pub const MAX_IGC_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;
pub const MAX_CSV_UPLOAD_BYTES: u64 = 10 * 1024 * 1024;

pub const NAME: &str = "flugbuech-api";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = "Paragliding flight book.";

// Config

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    /// Allowed CORS origin.
    ///
    /// - Set to `*` to allow CORS requests from all origins
    /// - Set to a specific origin to allow CORS requests from that origin
    pub cors_allow_origin: Option<String>,
}

// Index

#[get("/")]
async fn index() -> String {
    format!("{NAME} {VERSION}: https://github.com/dbrgn/flugbuech/")
}

// Handle missing DB

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
    "Service is not available. (Is the database up?)"
}

// Main

#[rocket::main]
async fn main() -> Result<()> {
    // Load env
    let _ = dotenvy::dotenv();

    // Parse args
    let args = Command::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .arg(
            Arg::new("migrate")
                .long("migrate")
                .action(ArgAction::SetTrue)
                .help("Run database migrations before starting"),
        )
        .get_matches();

    // Decide whether migrations should be run
    let migrate = args.get_flag("migrate");
    if migrate {
        println!("Running database migrations...");
        data::run_migrations().unwrap();
    }

    // Initialize application
    let app = rocket::build();

    // Extract additional config values
    let figment = app.figment();
    let config: Config = figment.extract().context("Could not extract config")?;

    // Attach fairings
    let app = app
        .attach(data::Database::fairing())
        .attach(cors::Cors::from_config(config.cors_allow_origin.as_deref()));

    // Register custom error catchers
    let app = app.register("/", catchers![service_not_available]);

    // Attach routes
    let app = app
        // API routes
        .mount(
            "/api/v1/",
            [
                routes![index],
                auth::api_routes(),
                profile::api_routes(),
                stats::api_routes(),
                locations::api_routes(),
                gliders::api_routes(),
                flights::api_routes(),
                process_igc::api_routes(),
                import_csv::api_routes(),
            ]
            .concat(),
        );

    // Launch app
    app.ignite()
        .await
        .context("Could not ignite app")?
        .launch()
        .await
        .context("Could not launch app")?;
    Ok(())
}
