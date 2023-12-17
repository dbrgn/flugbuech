//! Process IGC files and extract relevant information.

use std::io::{self, BufRead, BufReader, Cursor};

use flat_projection::{FlatPoint, FlatProjection};
use igc::{
    records::{HRecord, Record},
    util::RawPosition,
};
use num_traits::Float;
use rocket::{
    data::{Data, ToByteUnit},
    post,
    serde::json::Json,
};
use serde::Serialize;

use crate::{auth, data, models};

#[derive(Debug, PartialEq, Serialize)]
struct LatLng {
    lat: f64,
    lng: f64,
}

#[derive(Debug, PartialEq, Serialize)]
struct LaunchLandingInfo {
    pos: LatLng,
    alt: i16,
    time_hms: (u8, u8, u8),
    location_id: Option<i32>,
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub struct FlightInfo {
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

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum FlightInfoResult {
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

fn parse_igc(reader: impl BufRead, user: &models::User, db: &mut diesel::PgConnection) -> FlightInfoResult {
    log::info!("Parsing IGC file for user {}", user.id);

    // Split lines in IGC file
    //
    // NOTE: This will yield a vector of Vec<u8>. We cannot use `.lines()`
    //       directly since that will fail if the data is invalid UTF8, and we
    //       want to parse leniently in case of invalid UTF8 data.
    //       Unfortunately when splitting by '\n' there can still be remaining
    //       '\r' characters that must be trimmed later.
    let lines = match reader.split(b'\n').collect::<Result<Vec<Vec<u8>>, io::Error>>() {
        Ok(res) => res,
        Err(e) => {
            return FlightInfoResult::Error {
                msg: format!("I/O Error: {}", e),
            }
        }
    };

    // Prepare FlightInfo instance
    let mut info = FlightInfo::default();

    // Vector to collect track coordinates projected from WGS84 into a
    // cartesian coordinate system
    let mut projection: Option<FlatProjection<f64>> = None;
    let mut flight_path = FlatPointString::new();

    for line_bytes in &lines {
        let line = String::from_utf8_lossy(line_bytes);
        match Record::parse_line(line.trim()) {
            Ok(Record::H(h @ HRecord { mnemonic: "PLT", .. })) => {
                info.pilot = Some(h.data.trim().into());
            }
            Ok(Record::H(h @ HRecord { mnemonic: "GTY", .. })) => {
                info.glidertype = Some(h.data.trim().into());
            }
            Ok(Record::H(h @ HRecord { mnemonic: "SIT", .. })) => {
                info.site = Some(h.data.trim().into());
            }
            Ok(Record::H(h @ HRecord { mnemonic: "DTE", .. })) => {
                let string_val = h.data.trim();
                // Date formats:
                // - Skytraxx: DDMMYY
                // - XCTrack:  DDMMYY,[??]
                if string_val.len() == 6 || (string_val.len() == 9 && &string_val[6..7] == ",") {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        string_val.get(0..2).unwrap().parse::<u8>(),
                        string_val.get(2..4).unwrap().parse::<u8>(),
                        string_val.get(4..6).unwrap().parse::<u16>(),
                    ) {
                        let year = if year > 85 { 1900 + year } else { 2000 + year };
                        info.date_ymd = Some((year, month, day));
                    }
                } else {
                    log::warn!("Unexpected H record DTE format: {:?}", h);
                }
            }
            Ok(Record::B(b)) => {
                // Extract raw float coordinates
                let RawPosition {
                    lat: raw_lat,
                    lon: raw_lng,
                } = b.pos;
                let lat = raw_lat.into();
                let lng = raw_lng.into();
                let pos = LatLng { lat, lng };

                // Initialize projection with the first coordinate in the track
                if projection.is_none() {
                    projection = Some(FlatProjection::new(lng, lat));
                }

                // Project the coordinate onto a flat coordinate system
                let flat_point = projection.unwrap().project(lng, lat);
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
            }
            Ok(_rec) => {}
            Err(e) => {
                return FlightInfoResult::Error {
                    msg: format!("Error parsing lines: {:?}", e),
                }
            }
        }
    }
    info.track_distance = flight_path.length();

    // Find locations within 1000 meters of launch and landing
    let max_distance = 1000.0;
    if let Some(ref mut launch) = info.launch {
        launch.location_id =
            data::get_locations_around_point(db, user, launch.pos.lat, launch.pos.lng, max_distance)
                .get(0)
                .map(|location| location.id);
    }
    if let Some(ref mut landing) = info.landing {
        landing.location_id =
            data::get_locations_around_point(db, user, landing.pos.lat, landing.pos.lng, max_distance)
                .get(0)
                .map(|location| location.id);
    }

    FlightInfoResult::Success(info)
}

/// Process IGC file, return parsed data.
///
/// This endpoint is meant to be called from a XmlHttpRequest.
#[post(
    "/flights/add/process_igc",
    format = "application/octet-stream",
    data = "<data>"
)]
pub async fn process_igc(
    data: Data<'_>,
    user: auth::AuthUser,
    database: data::Database,
) -> Json<FlightInfoResult> {
    let user = user.into_inner();

    // Open IGC file
    let igc_bytes = match data.open(crate::MAX_UPLOAD_BYTES.bytes()).into_bytes().await {
        Ok(capped_vec) if capped_vec.is_complete() => capped_vec.into_inner(),
        Ok(_) => {
            return Json(FlightInfoResult::Error {
                msg: "Too many bytes received while reading IGC data".into(),
            })
        }
        Err(e) => {
            log::error!("Error while reading IGC data: {}", e);
            return Json(FlightInfoResult::Error {
                msg: "Error while reading IGC data".into(),
            });
        }
    };
    let buf_reader = BufReader::new(Cursor::new(igc_bytes));

    // Process data
    Json(database.run(move |db| parse_igc(buf_reader, &user, db)).await)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::DbTestContext;

    use super::*;

    fn process(data: &str) -> Result<FlightInfo, String> {
        let ctx = DbTestContext::new();
        let reader = BufReader::new(Cursor::new(data));
        let result = parse_igc(reader, &ctx.testuser1.user, &mut ctx.force_get_conn());
        match result {
            FlightInfoResult::Success(info) => Ok(info),
            FlightInfoResult::Error { msg } => Err(msg),
        }
    }

    #[test]
    fn parse_simple_igc() {
        let data = include_str!("../testdata/skytraxx.igc");
        let info = process(data).unwrap();
        assert_eq!(info.pilot, Some("Danilo".to_string()));
        assert_eq!(info.glidertype, Some("Epsilon 8".to_string()));
        assert_eq!(info.site, Some("Hitzeggen".to_string()));
        assert_eq!(info.date_ymd, Some((2019, 7, 22)));
        assert_eq!(
            info.launch,
            Some(LaunchLandingInfo {
                pos: LatLng {
                    lat: 46.71985,
                    lng: 9.149533333333334
                },
                alt: 1568,
                time_hms: (13, 42, 26),
                location_id: None
            })
        );
        assert_eq!(
            info.landing,
            Some(LaunchLandingInfo {
                pos: LatLng {
                    lat: 46.70665,
                    lng: 9.153933333333333,
                },
                alt: 1300,
                time_hms: (13, 46, 7),
                location_id: None,
            })
        );
        assert!(
            info.track_distance > 1.98988,
            "Track distance is {:?}, not >1.98988",
            info.track_distance
        );
        assert!(
            info.track_distance < 1.98989,
            "Track distance is {:?}, not <1.98989",
            info.track_distance
        );
    }

    /// Parse IGC data with only two lines: pilot name and site.
    #[test]
    fn parse_minimal() {
        let data = "HFPLTPILOT: Chrigel Maurer\nHPSITSITE: Interlaken";
        let info = process(data).unwrap();
        assert_eq!(
            info,
            FlightInfo {
                pilot: Some("Chrigel Maurer".to_string()),
                site: Some("Interlaken".to_string()),
                ..Default::default()
            }
        );
    }

    /// Parse IGC data with windows line endings.
    #[test]
    fn regression_27_cr_endings() {
        let data = "HFPLTPILOT: Chrigel Maurer\r\nI023636LAD3737LOD\r\n";
        let info = process(data).unwrap();
        assert_eq!(
            info,
            FlightInfo {
                pilot: Some("Chrigel Maurer".to_string()),
                ..Default::default()
            }
        );
    }

    /// Handle XCTrack date format.
    #[test]
    fn regression_30_xctrack_date_format() {
        let data = "HFDTEDATE:280719,02";
        let info = process(data).unwrap();
        assert_eq!(
            info,
            FlightInfo {
                date_ymd: Some((2019, 7, 28)),
                ..Default::default()
            }
        );
    }
}
