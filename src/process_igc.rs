//! Process IGC files and extract relevant information.

use std::io::{self, BufRead, BufReader, Read};

use flat_projection::{FlatPoint, FlatProjection};
use igc::records::{HRecord, Record};
use igc::util::RawPosition;
use num_traits::Float;
use rocket::data::Data;
use rocket::post;
use rocket_contrib::json::Json;
use serde::Serialize;

use crate::{auth, data};

#[derive(Debug, PartialEq, Serialize)]
struct LatLon {
    lat: f64,
    lon: f64,
}

#[derive(Debug, PartialEq, Serialize)]
struct LaunchLandingInfo {
    pos: LatLon,
    alt: i16,
    time_hms: (u8, u8, u8),
    location_id: Option<i32>,
}

impl LaunchLandingInfo {
    fn seconds_since_midnight(&self) -> u32 {
        let hs = u32::from(self.time_hms.0) * 24 * 60;
        let ms = u32::from(self.time_hms.1) * 60;
        let ss = u32::from(self.time_hms.2);
        hs + ms + ss
    }
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub(crate) struct FlightInfo {
    /// Name of the pilot, as configured in the flight instrument.
    pilot: Option<String>,
    /// Name of the glider, as configured in the flight instrument.
    glidertype: Option<String>,
    /// Name of the launch site, as configured in the flight instrument.
    site: Option<String>,
    /// Date of flight (YYYY, MM, DD).
    date_ymd: Option<(u16, u8, u8)>,
    /// Lauch infos.
    launch: Option<LaunchLandingInfo>,
    /// Landing infos.
    landing: Option<LaunchLandingInfo>,
    /// Track length in kilometers.
    track_distance: f64,
}

impl FlightInfo {
    fn duration(&self) -> Option<u32> {
        if let (Some(launch), Some(landing)) = (&self.launch, &self.landing) {
            let launch_seconds = launch.seconds_since_midnight();
            let landing_seconds = landing.seconds_since_midnight();
            if landing_seconds > launch_seconds {
                Some(landing_seconds - launch_seconds)
            } else {
                Some(86400 - launch_seconds + landing_seconds)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum FlightInfoResult {
    Success(FlightInfo),
    Error { msg: String },
}

#[derive(Debug, PartialEq, Clone)]
struct FlatPointString<T>(Vec<FlatPoint<T>>);

impl<T: Float> FlatPointString<T> {
    fn new() -> Self {
        Self(vec![])
    }

    fn add_point(&mut self, point: FlatPoint<T>) {
        self.0.push(point);
    }

    /// Return the flight path length in km.
    fn length(&self) -> T {
        self.0
            .windows(2)
            .map(|pair| pair[0].distance(&pair[1]))
            .fold(T::zero(), |total, distance| total + distance)
    }
}

/// Process IGC file, return parsed data.
///
/// This endpoint is meant to be called from a XmlHttpRequest.
#[post(
    "/flights/add/process_igc",
    format = "application/octet-stream",
    data = "<data>"
)]
pub(crate) fn process_igc(data: Data, user: auth::AuthUser, db: data::Database) -> Json<FlightInfoResult> {
    let user = user.into_inner();

    // Open IGC file
    let reader = data.open().take(crate::MAX_UPLOAD_BYTES);
    let buf_reader = BufReader::new(reader);
    let lines = match buf_reader.split(b'\n').collect::<Result<Vec<Vec<u8>>, io::Error>>() {
        Ok(res) => res,
        Err(e) => {
            return Json(FlightInfoResult::Error {
                msg: format!("I/O Error: {}", e),
            })
        },
    };

    // Prepare FlightInfo instance
    let mut info = FlightInfo::default();

    // Vector to collect track coordinates projected from WGS84 into a
    // cartesian coordinate system
    let mut projection: Option<FlatProjection<f64>> = None;
    let mut flight_path = FlatPointString::new();

    for line_bytes in &lines {
        let line = String::from_utf8_lossy(line_bytes);
        match Record::parse_line(&line) {
            Ok(Record::H(h @ HRecord { mnemonic: "PLT", .. })) => {
                info.pilot = Some(h.data.trim().into());
            },
            Ok(Record::H(h @ HRecord { mnemonic: "GTY", .. })) => {
                info.glidertype = Some(h.data.trim().into());
            },
            Ok(Record::H(h @ HRecord { mnemonic: "SIT", .. })) => {
                info.site = Some(h.data.trim().into());
            },
            Ok(Record::H(h @ HRecord { mnemonic: "DTE", .. })) => {
                let string_val = h.data.trim();
                if string_val.len() == 6 {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        string_val.get(0..2).unwrap().parse::<u8>(),
                        string_val.get(2..4).unwrap().parse::<u8>(),
                        string_val.get(4..6).unwrap().parse::<u16>(),
                    ) {
                        let year = if year > 85 { 1900 + year } else { 2000 + year };
                        info.date_ymd = Some((year, month, day));
                    }
                }
            },
            Ok(Record::B(b)) => {
                println!("{}: {} (GPS) / {} (Baro)", b.timestamp, b.gps_alt, b.pressure_alt);

                // Extract raw float coordinates
                let RawPosition {
                    lat: raw_lat,
                    lon: raw_lon,
                } = b.pos;
                let lat = raw_lat.into();
                let lon = raw_lon.into();
                let pos = LatLon { lat, lon };

                // Initialize projection with the first latitude in the track
                if projection.is_none() {
                    projection = Some(FlatProjection::new(lat));
                }

                // Project the coordinate onto a flat coordinate system
                let flat_point = projection.unwrap().project(lon, lat);
                flight_path.add_point(flat_point);

                // TODO: More elaborate launch detection using altitude
                if info.launch.is_none() {
                    // Create launch info
                    info.launch = Some(LaunchLandingInfo {
                        pos,
                        alt: b.gps_alt,
                        time_hms: (b.timestamp.hours, b.timestamp.minutes, b.timestamp.seconds),
                        location_id: None,
                    });
                } else {
                    info.landing = Some(LaunchLandingInfo {
                        pos,
                        alt: b.gps_alt,
                        time_hms: (b.timestamp.hours, b.timestamp.minutes, b.timestamp.seconds),
                        location_id: None,
                    });
                }
            },
            Ok(_rec) => {},
            Err(e) => {
                return Json(FlightInfoResult::Error {
                    msg: format!("Error parsing lines: {:?}", e),
                })
            },
        }
    }
    info.track_distance = flight_path.length();

    // Find locations within 1000 meters of launch and landing
    let max_distance = 1000.0;
    if let Some(ref mut launch) = info.launch {
        launch.location_id =
            data::get_locations_around_point(&db, &user, launch.pos.lat, launch.pos.lon, max_distance)
                .iter()
                .next()
                .map(|location| location.id);
    }
    if let Some(ref mut landing) = info.landing {
        landing.location_id =
            data::get_locations_around_point(&db, &user, landing.pos.lat, landing.pos.lon, max_distance)
                .iter()
                .next()
                .map(|location| location.id);
    }

    println!("Info: {:#?}", info);
    println!("Flight duration: {:?} seconds", info.duration());

    // Create a flight
    //let flight = data::create_flight(&db, models::NewFlight {
    //    user_id: 0,
    //    ..Default::default()
    //});
    //println!("{:?}", flight);

    Json(FlightInfoResult::Success(info))
}
