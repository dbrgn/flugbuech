//! Submit form.

use std::fmt;

use base64;
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{DateTime, Utc};
use rocket::http::RawStr;
use rocket::request::{Form, FromForm, FromFormValue};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_contrib::templates::Template;
use serde::Serialize;

use crate::{auth, data, models};

/// A combined Option / Result type.
#[derive(Debug)]
enum OptionResult<T> {
    None,
    Ok(T),
    Err(String),
}

impl<T> OptionResult<T> {
    fn into_result(self) -> Result<Option<T>, String> {
        match self {
            Self::None => Ok(None),
            Self::Ok(v) => Ok(Some(v)),
            Self::Err(e) => Err(e),
        }
    }
}

impl<'v> FromFormValue<'v> for OptionResult<NaiveDate> {
    type Error = String;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        NaiveDate::parse_from_str(form_value, "%Y-%m-%d")
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid date ({}): {}", form_value, e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<NaiveTime> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        let mut decoded = match form_value.url_decode() {
            Ok(val) => val,
            Err(e) => return Ok(OptionResult::Err(format!("Could not urldecode value: {}", e))),
        };
        if decoded.len() < 8 {
            decoded.push_str(":00");
        }
        NaiveTime::parse_from_str(&decoded, "%H:%M:%S")
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid time ({}): {}", decoded, e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<i32> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        form_value
            .parse()
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid integer: {}", e))))
    }
}

impl<'v> FromFormValue<'v> for OptionResult<f32> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        form_value
            .parse()
            .map(OptionResult::Ok)
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid integer: {}", e))))
    }
}

/// Data that is passed in as URL safe Base64
struct Base64Data(Vec<u8>);

impl fmt::Debug for Base64Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() > 10 {
            write!(
                f,
                "Base64Data([{}, {}, {}, ...{} more bytes)",
                self.0[0],
                self.0[1],
                self.0[2],
                self.0.len() - 3,
            )
        } else {
            write!(f, "Base64Data({:?})", self.0)
        }
    }
}

impl<'v> FromFormValue<'v> for OptionResult<Base64Data> {
    type Error = !;
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        if form_value.trim().is_empty() {
            return Ok(OptionResult::None);
        }
        base64::decode_config(form_value, base64::URL_SAFE)
            .map(|vec| OptionResult::Ok(Base64Data(vec)))
            .or_else(|e| Ok(OptionResult::Err(format!("Invalid base64 data: {}", e))))
    }
}

#[derive(FromForm, Debug)]
pub(crate) struct SubmitForm {
    igc_data: OptionResult<Base64Data>,
    number: OptionResult<i32>,
    aircraft: Option<i32>,
    launch_site: Option<i32>,
    landing_site: Option<i32>,
    launch_date: OptionResult<NaiveDate>,
    launch_time: OptionResult<NaiveTime>,
    landing_time: OptionResult<NaiveTime>,
    track_distance: OptionResult<f32>,
    xcontest_tracktype: String,
    xcontest_distance: OptionResult<f32>,
    xcontest_url: String,
    comment: String,
    video_url: String,
}

#[derive(Serialize)]
struct SubmitContext {
    user: models::User,
    aircraft_list: Vec<models::Aircraft>,
    locations: Vec<models::Location>,
    error_msg: Option<String>,
}

#[get("/flights/add")]
pub(crate) fn submit_form(user: auth::AuthUser, db: data::Database) -> Template {
    let user = user.into_inner();
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);
    let context = SubmitContext {
        user,
        aircraft_list,
        locations,
        error_msg: None,
    };
    Template::render("submit", context)
}

#[get("/flights/add", rank = 2)]
pub(crate) fn submit_form_nologin() -> Redirect {
    Redirect::to("/auth/login")
}

#[post("/flights/add", data = "<data>")]
pub(crate) fn submit(
    user: auth::AuthUser,
    db: data::Database,
    data: Option<Form<SubmitForm>>,
) -> Result<Redirect, Template> {
    let user = user.into_inner();
    let aircraft_list = data::get_aircraft_for_user(&db, &user);
    let locations = data::get_locations_for_user(&db, &user);

    macro_rules! fail {
        ($msg:expr) => {{
            let error_msg = Some($msg.into());
            let ctx = SubmitContext {
                user,
                aircraft_list,
                locations,
                error_msg,
            };
            return Err(Template::render("submit", ctx));
        }};
    }

    macro_rules! none_if_empty {
        ($val:expr) => {
            if $val.trim().is_empty() {
                None
            } else {
                Some($val)
            }
        };
    }

    if let Some(Form(SubmitForm {
        igc_data: form_igc_data,
        number: form_number,
        aircraft: form_aircraft,
        launch_site: form_launch_site,
        landing_site: form_landing_site,
        launch_date: form_launch_date,
        launch_time: form_launch_time,
        landing_time: form_landing_time,
        track_distance: form_track_distance,
        xcontest_tracktype: form_xcontest_tracktype,
        xcontest_distance: form_xcontest_distance,
        xcontest_url: form_xcontest_url,
        comment: form_comment,
        video_url: form_video_url,
    })) = data
    {
        // TODO
        if let OptionResult::Err(ref e) = form_igc_data {
            fail!(format!("IGC File: {}", e));
        };

        // Extract basic model data
        let number = match form_number.into_result() {
            Ok(name) => name,
            Err(_) => fail!("Invalid flight number"),
        };
        let user_id = user.id;
        let aircraft_id = form_aircraft;

        // Extract date and time
        let mut date_parts = 0;
        let launch_date_naive = match form_launch_date.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Launch date: {}", e)),
        };
        let launch_time_naive = match form_launch_time.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Launch time: {}", e)),
        };
        let landing_time_naive = match form_landing_time.into_result() {
            Ok(val) => {
                date_parts += 1;
                val
            },
            Err(e) => fail!(format!("Landing time: {}", e)),
        };
        if date_parts < 0 && date_parts < 3 {
            fail!("If you specify launch date, launch time or landing time, then the other two values must be provided as well");
        }

        // Extract launch / landing information
        let launch_at = form_launch_site;
        let landing_at = form_landing_site;
        let launch_time = launch_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });
        let landing_time = landing_time_naive.map(|time| {
            let ndt = NaiveDateTime::new(launch_date_naive.unwrap(), time);
            DateTime::from_utc(ndt, Utc) // TODO: Timezone
        });

        // Extract track information
        let track_distance = match form_track_distance.into_result() {
            Ok(val) => val,
            Err(_) => fail!("Invalid track distance"),
        };
        let xcontest_tracktype = none_if_empty!(form_xcontest_tracktype);
        let xcontest_distance = match form_xcontest_distance.into_result() {
            Ok(val) => val,
            Err(_) => fail!("Invalid XContest distance"),
        };
        let xcontest_url = none_if_empty!(form_xcontest_url);

        // Extract other information
        let comment = none_if_empty!(form_comment);
        let video_url = none_if_empty!(form_video_url);

        // Create model
        let flight = models::NewFlight {
            number,
            user_id,
            aircraft_id,
            launch_at,
            landing_at,
            launch_time,
            landing_time,
            track_distance,
            xcontest_tracktype,
            xcontest_distance,
            xcontest_url,
            comment,
            video_url,
        };
        // TODO: Error handling
        data::create_flight(&db, flight);

        Ok(Redirect::to("/flights/"))
    } else {
        fail!("Invalid form, could not parse data. Note: Only IGC files up to ~2 MiB can be uploaded.");
    }
}
