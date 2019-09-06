//! Process IGC files and extract relevant information.

use std::io::{self, Read, BufRead, BufReader};

use igc::records::{Record, HRecord};
use igc::util::RawPosition;
use rocket::post;
use rocket::data::Data;
use rocket_contrib::json::Json;
use serde::Serialize;


#[derive(Debug, PartialEq, Serialize)]
struct LatLon {
    lat: f64,
    lon: f64,
}

impl From<RawPosition> for LatLon {
    fn from(pos: RawPosition) -> Self {
        Self {
            lat: pos.lat.into(),
            lon: pos.lon.into(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct LaunchLandingInfo {
    pos: LatLon,
    alt: i16,
    time_hms: (u8, u8, u8),
}

impl LaunchLandingInfo {
    fn seconds_since_midnight(&self) -> u32 {
        u32::from(self.time_hms.0) * 24 * 60 +
        u32::from(self.time_hms.1) * 60 +
        u32::from(self.time_hms.2)
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
    Error(String),
}

/// Process IGC file, return parsed data.
///
/// This endpoint is meant to be called from a XmlHttpRequest.
#[post("/submit/process_igc", format = "application/octet-stream", data = "<data>")]
pub(crate) fn process_igc(data: Data) -> Json<FlightInfoResult> {
    // Read IGC file, create FlightInfo instance
    let reader = data.open().take(crate::MAX_UPLOAD_BYTES);
    let buf_reader = BufReader::new(reader);
    let lines = match buf_reader.lines().collect::<Result<Vec<String>, io::Error>>() {
        Ok(res) => res,
        Err(e) => return Json(FlightInfoResult::Error(format!("I/O Error: {}", e))),
    };
    let mut info = FlightInfo::default();
    for line in &lines {
        match Record::parse_line(&line) {
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
                if string_val.len() == 6 {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        string_val.get(0..2).unwrap().parse::<u8>(),
                        string_val.get(2..4).unwrap().parse::<u8>(),
                        string_val.get(4..6).unwrap().parse::<u16>(),
                    ) {
                        info.date_ymd = Some((
                            if year > 85 { 1900 + year } else { 2000 + year },
                            month,
                            day
                        ));
                    }
                }
            }
            Ok(Record::B(b)) => {
                println!("{}: {} (GPS) / {} (Baro)", b.timestamp, b.gps_alt, b.pressure_alt);
                if info.launch.is_none() {
                    info.launch = Some(LaunchLandingInfo {
                        pos: b.pos.into(),
                        alt: b.gps_alt,
                        time_hms: (b.timestamp.hours, b.timestamp.minutes, b.timestamp.seconds),
                    });
                } else {
                    info.landing = Some(LaunchLandingInfo {
                        pos: b.pos.into(),
                        alt: b.gps_alt,
                        time_hms: (b.timestamp.hours, b.timestamp.minutes, b.timestamp.seconds),
                    });
                }
            }
            Ok(_rec) => {},
            Err(e) => return Json(FlightInfoResult::Error(format!("Error parsing lines: {:?}", e))),
        }
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
