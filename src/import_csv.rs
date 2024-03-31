// API types

use std::{collections::HashSet, io::Cursor};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use diesel::PgConnection;
use log::{error, info};
use rocket::{data::ToByteUnit, post, routes, serde::json::Json, Data, Route};
use serde::{Deserialize, Serialize};

use crate::{auth, data, models::User, responders::ApiError, xcontest::is_valid_tracktype};

// API types

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    message: String,
    csv_row: Option<usize>,
    field: Option<String>,
}

impl Message {
    fn without_row(message: impl Into<String>) -> Self {
        Message {
            message: message.into(),
            ..Default::default()
        }
    }

    fn for_row(csv_row: usize, message: impl Into<String>) -> Self {
        Message {
            message: message.into(),
            csv_row: Some(csv_row),
            ..Default::default()
        }
    }

    fn for_field(csv_row: usize, field: impl Into<String>, message: impl Into<String>) -> Self {
        Message {
            message: message.into(),
            csv_row: Some(csv_row),
            field: Some(field.into()),
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvAnalyzeResult {
    warnings: Vec<Message>,
    errors: Vec<Message>,
    flights: Vec<ApiCsvFlightPreview>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCsvFlightPreview {
    /// The row number in the CSV (starting with 1)
    pub csv_row: usize,
    /// The user-defined flight number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i32>,
    /// The glider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glider_id: Option<i32>,
    /// Launch location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_at: Option<i32>,
    /// Landing location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_at: Option<i32>,
    /// Time of launch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_time: Option<DateTime<Utc>>,
    /// Time of landing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_time: Option<DateTime<Utc>>,
    /// GPS track length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_distance: Option<f32>,
    /// XContest tracktype (free_flight, flat_triangle or fai_triangle)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xcontest_tracktype: Option<String>,
    /// XContest distance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xcontest_distance: Option<f32>,
    /// XContest URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xcontest_url: Option<String>,
    /// Comment your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Link to a video of your flight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    /// Whether you hiked up to launch
    pub hikeandfly: bool,
}

// Helper types

static VALID_HEADERS: [&'static str; 14] = [
    "number",
    "date",
    "glider",
    "launch_site",
    "launch_time_utc",
    "landing_site",
    "landing_time_utc",
    "track_distance",
    "hikeandfly",
    "comment",
    "xcontest_url",
    "xcontest_tracktype",
    "xcontest_scored_distance",
    "video_url",
];

#[derive(Debug, Deserialize)]
struct CsvRecord {
    number: Option<i32>,
    date: Option<String>,
    glider: Option<String>,
    launch_site: Option<String>,
    launch_time_utc: Option<String>,
    landing_site: Option<String>,
    landing_time_utc: Option<String>,
    track_distance: Option<f32>,
    hikeandfly: Option<bool>,
    comment: Option<String>,
    xcontest_url: Option<String>,
    xcontest_tracktype: Option<String>,
    xcontest_scored_distance: Option<f32>,
    video_url: Option<String>,
}

// API endpoints

/// Process a CSV file
///
/// The `mode` GET parameter is required:
///
/// - analyze: Process and analyze the CSV data, but don't store it yet
/// - import: Process and store CSV data
#[post(
    "/flights/add/import_csv?<mode>",
    format = "application/octet-stream",
    data = "<data>"
)]
pub async fn process_csv(
    user: auth::AuthUser,
    database: data::Database,
    mode: &'_ str,
    data: Data<'_>,
) -> Result<Json<CsvAnalyzeResult>, ApiError> {
    info!("Processing CSV with mode '{mode}'");
    let user = user.into_inner();

    // Validate mode
    if !["analyze", "import"].contains(&mode) {
        return Err(ApiError::InvalidData {
            message: format!("Invalid mode: {mode}"),
        });
    }

    // Right now the csv crate does not support async streams, so instead we'll collect the CSV data
    // into a vec. This is fine, given that the max upload size is 10 MiB.
    let csv_bytes = match data.open(crate::MAX_CSV_UPLOAD_BYTES.bytes()).into_bytes().await {
        Ok(capped_bytes) => {
            assert!(
                capped_bytes.is_complete(),
                "Expected capped bytes to be complete, but is_complete() returned false"
            );
            capped_bytes.into_inner()
        }
        Err(e) => {
            error!("Failed to read CSV data: {e:?}");
            return Err(ApiError::IoError {
                message: format!("Failed to read CSV data"),
            });
        }
    };

    // Process and analyze the CSV file
    let analyze_result = database.run(move |db| analyze_csv(csv_bytes, &user, db)).await;

    if mode == "import" {
        todo!("Import not yet implemented");
    }

    Ok(Json(analyze_result))
}

/// Return vec of all API routes.
pub fn api_routes() -> Vec<Route> {
    routes![process_csv]
}

// Helpers

/// Process, analyze and return (but don't save) flights from CSV
fn analyze_csv(csv_bytes: Vec<u8>, user: &User, conn: &mut PgConnection) -> CsvAnalyzeResult {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut flights = vec![];

    macro_rules! fail {
        ($msg:expr, $csv_row:expr) => {{
            errors.push(Message {
                message: $msg.into(),
                csv_row: $csv_row,
                field: None,
            });
            return CsvAnalyzeResult {
                warnings,
                errors,
                flights,
            };
        }};
    }

    // Create CSV reader
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .quote(b'"')
        .from_reader(Cursor::new(csv_bytes));

    // Parse and validate headers
    let valid_header_set = HashSet::from(VALID_HEADERS);
    match reader.headers() {
        Ok(headers) => {
            if headers.is_empty() {
                fail!("CSV does not contain any columns", None);
            }

            let given_header_set = headers.into_iter().collect::<HashSet<_>>();
            if valid_header_set.is_disjoint(&given_header_set) {
                fail!(
                    format!(
                        "CSV header fields ({}) don't contain any valid header",
                        headers.iter().collect::<Vec<_>>().join(","),
                    ),
                    None
                );
            }
            let mut unknown_fields = given_header_set
                .difference(&valid_header_set)
                .cloned()
                .collect::<Vec<_>>();
            if !unknown_fields.is_empty() {
                unknown_fields.sort();
                warnings.push(Message::without_row(format!(
                    "Some CSV header fields are unknown and will be ignored: {}",
                    unknown_fields.join(","),
                )));
            }
        }
        Err(e) => fail!(format!("Error while reading headers from CSV: {e}"), None),
    };

    // Get user's gliders
    let gliders: Vec<(i32, String)> = data::get_gliders_for_user(conn, user)
        .into_iter()
        .map(|glider| (glider.id, format!("{} {}", glider.manufacturer, glider.model)))
        .collect();

    // Get user's locations
    let locations: Vec<(i32, String)> = data::get_locations_for_user(conn, user)
        .into_iter()
        .map(|location| (location.id, location.name))
        .collect();

    // Parse and validate records
    for (row_number, result) in reader.deserialize().enumerate() {
        let row_number1 = row_number + 1;

        let record: CsvRecord = match result {
            Ok(r) => r,
            Err(e) => {
                warnings.push(Message::for_row(
                    row_number1,
                    format!("Error while reading record from CSV: {e}"),
                ));
                continue;
            }
        };

        // Prepare flight preview struct
        let mut flight = ApiCsvFlightPreview {
            csv_row: row_number1,
            number: record.number,
            track_distance: record.track_distance,
            comment: record.comment.clone(),
            video_url: record.video_url.clone(),
            hikeandfly: record.hikeandfly.unwrap_or(false),
            ..Default::default()
        };

        flight_process_glider(&record, row_number1, &mut flight, &gliders, &mut warnings);
        flight_process_locations(&record, row_number1, &mut flight, &locations, &mut warnings);
        flight_process_date_time(&record, row_number1, &mut flight, &mut warnings);
        flight_process_xcontest_info(&record, row_number1, &mut flight, &mut warnings);

        flights.push(flight);
    }

    if flights.is_empty() {
        fail!("CSV is empty", None);
    }

    CsvAnalyzeResult {
        warnings,
        errors,
        flights,
    }
}

fn flight_process_glider(
    record: &CsvRecord,
    row_number1: usize,
    flight: &mut ApiCsvFlightPreview,
    gliders: &Vec<(i32, String)>,
    warnings: &mut Vec<Message>,
) {
    if let Some(glider) = record.glider.as_ref() {
        flight.glider_id = gliders
            .iter()
            .find_map(|(id, name)| if name == glider { Some(*id) } else { None });
        if flight.glider_id.is_none() {
            warnings.push(Message::for_field(
                row_number1,
                "glider_id",
                format!("Could not find glider with name \"{glider}\" in your list of gliders"),
            ));
        }
    }
}

fn flight_process_locations(
    record: &CsvRecord,
    row_number1: usize,
    flight: &mut ApiCsvFlightPreview,
    locations: &Vec<(i32, String)>,
    warnings: &mut Vec<Message>,
) {
    // Look up launch and landing location
    if let Some(launch_site) = record.launch_site.as_ref() {
        flight.launch_at = locations
            .iter()
            .find_map(|(id, name)| if name == launch_site { Some(*id) } else { None });
        if flight.launch_at.is_none() {
            warnings.push(Message::for_field(
                row_number1,
                "launch_at",
                format!("Could not find launch site with name \"{launch_site}\" in your list of locations"),
            ));
        }
    }
    if let Some(landing_site) = record.landing_site.as_ref() {
        flight.landing_at = locations
            .iter()
            .find_map(|(id, name)| if name == landing_site { Some(*id) } else { None });
        if flight.landing_at.is_none() {
            warnings.push(Message::for_field(
                row_number1,
                "landing_at",
                format!("Could not find landing site with name \"{landing_site}\" in your list of locations"),
            ));
        }
    }
}

fn flight_process_date_time(
    record: &CsvRecord,
    row_number1: usize,
    flight: &mut ApiCsvFlightPreview,
    warnings: &mut Vec<Message>,
) {
    let date_parts = record.date.as_ref().map(|_| 1).unwrap_or_default()
        + record.launch_time_utc.as_ref().map(|_| 1).unwrap_or_default()
        + record.landing_time_utc.as_ref().map(|_| 1).unwrap_or_default();
    if date_parts > 0 && date_parts < 3 {
        warnings.push(
            Message::for_row(
                row_number1,
                "If you specify date, launch time or landing time, then the other two values must be provided as well",
        ));
    }
    if let (Some(date), Some(launch_time), Some(landing_time)) = (
        record.date.as_ref().and_then(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| {
                    warnings.push(Message::for_row(
                        row_number1,
                        format!("Invalid ISO date: {}", date_str),
                    ))
                })
                .ok()
        }),
        record.launch_time_utc.as_ref().and_then(|time_str| {
            NaiveTime::parse_from_str(time_str, "%H:%M:%S")
                .map_err(|_| {
                    warnings.push(Message::for_field(
                        row_number1,
                        "launch_time",
                        format!("Invalid launch time: {}", time_str),
                    ))
                })
                .ok()
        }),
        record.landing_time_utc.as_ref().and_then(|time_str| {
            NaiveTime::parse_from_str(time_str, "%H:%M:%S")
                .map_err(|_| {
                    warnings.push(Message::for_field(
                        row_number1,
                        "landing_time",
                        format!("Invalid landing time: {}", time_str),
                    ))
                })
                .ok()
        }),
    ) {
        flight.launch_time = Some(DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(date, launch_time),
            Utc,
        ));
        flight.landing_time = Some(DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(date, landing_time),
            Utc,
        ));
    }
}

fn flight_process_xcontest_info(
    record: &CsvRecord,
    row_number1: usize,
    flight: &mut ApiCsvFlightPreview,
    warnings: &mut Vec<Message>,
) {
    if let Some(tracktype) = record.xcontest_tracktype.as_ref() {
        if !is_valid_tracktype(tracktype) {
            warnings.push(Message::for_field(
                row_number1,
                "xcontest_tracktype",
                format!("Invalid XContest tracktype: {tracktype}"),
            ));
        } else {
            flight.xcontest_tracktype = Some(tracktype.into());
        }
    }
    flight.xcontest_distance = record.xcontest_scored_distance;
    if let Some(url) = record.xcontest_url.as_ref() {
        if url.starts_with("https://") {
            flight.xcontest_url = Some(url.into());
        } else if url.starts_with("http://") {
            flight.xcontest_url = Some(format!("https://{}", &url[7..]));
        } else {
            warnings.push(Message::for_field(
                row_number1,
                "xcontest_url",
                format!("XContest URL must start with https:// or http://"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{NewGlider, NewLocation},
        test_utils::{utc_datetime, DbTestContext},
    };

    use super::*;

    fn analyze(csv: &'static str, ctx: Option<DbTestContext>) -> CsvAnalyzeResult {
        let ctx = ctx.unwrap_or_else(|| DbTestContext::new());
        return analyze_csv(
            csv.as_bytes().to_vec(),
            &ctx.testuser1.user,
            &mut ctx.force_get_conn(),
        );
    }

    fn empty_vec() -> Vec<Message> {
        vec![]
    }

    #[test]
    fn analyze_empty_csv() {
        let result = analyze("", None);
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(
            result.errors,
            vec![Message::without_row("CSV does not contain any columns")]
        );
        assert_eq!(result.flights, vec![]);
    }

    #[test]
    fn analyze_csv_without_valid_headers() {
        let result = analyze("a,c,b\n1,2,3", None);
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(
            result.errors,
            vec![Message::without_row(
                "CSV header fields (a,c,b) don't contain any valid header"
            )]
        );
    }

    #[test]
    fn analyze_empty_csv_with_some_valid_headers() {
        let result = analyze("a,number,c,b\n", None);
        assert_eq!(
            result.warnings,
            vec![Message::without_row(
                "Some CSV header fields are unknown and will be ignored: a,b,c"
            )],
        );
        assert_eq!(result.errors, vec![Message::without_row("CSV is empty")]);
    }

    #[test]
    fn analyze_csv_empty_glider() {
        let result = analyze("number,glider\n42,", None);
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].glider_id, None);
    }

    #[test]
    fn analyze_csv_unknown_glider() {
        let result = analyze("number,glider\n42,Advance Omega ULS", None);
        assert_eq!(
            result.warnings,
            vec![Message::for_field(
                1,
                "glider_id",
                "Could not find glider with name \"Advance Omega ULS\" in your list of gliders",
            )]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].glider_id, None);
    }

    #[test]
    fn analyze_csv_known_glider() {
        let ctx = DbTestContext::new();
        let glider = data::create_glider(
            &mut ctx.force_get_conn(),
            NewGlider {
                user_id: ctx.testuser1.user.id,
                manufacturer: "Advance".into(),
                model: "Omega ULS".into(),
                since: None,
                until: None,
                source: None,
                cost: None,
                comment: None,
            },
        )
        .unwrap();

        let result = analyze("number,glider\n42,Advance Omega ULS", Some(ctx));
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].glider_id, Some(glider.id));
    }

    #[test]
    fn analyze_csv_unknown_locations() {
        let result = analyze("number,launch_site,landing_site\n42,Züri,Rappi", None);
        assert_eq!(
            result.warnings,
            vec![
                Message::for_field(
                    1,
                    "launch_at",
                    "Could not find launch site with name \"Züri\" in your list of locations",
                ),
                Message::for_field(
                    1,
                    "landing_at",
                    "Could not find landing site with name \"Rappi\" in your list of locations",
                ),
            ]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].launch_at, None);
        assert_eq!(result.flights[0].landing_at, None);
    }

    #[test]
    fn analyze_csv_known_locations() {
        let ctx = DbTestContext::new();
        let location1 = data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Züri".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );
        let location2 = data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Rappi".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );

        let result = analyze("number,launch_site,landing_site\n42,Züri,Rappi", Some(ctx));
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].launch_at, Some(location1.id));
        assert_eq!(result.flights[0].landing_at, Some(location2.id));
    }

    #[test]
    fn analyze_csv_permission_checks() {
        let ctx = DbTestContext::new();

        // Note: All entities match but are owned by another user
        data::create_glider(
            &mut ctx.force_get_conn(),
            NewGlider {
                user_id: ctx.testuser2.user.id,
                manufacturer: "Advance".into(),
                model: "Alpha".into(),
                since: None,
                until: None,
                source: None,
                cost: None,
                comment: None,
            },
        )
        .unwrap();
        data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Züri".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser2.user.id,
                geog: None,
            },
        );
        data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Rappi".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser2.user.id,
                geog: None,
            },
        );

        let result = analyze(
            "number,glider,launch_site,landing_site\n42,Advance Alpha,Züri,Rappi",
            Some(ctx),
        );
        assert_eq!(
            result.warnings,
            vec![
                Message::for_field(
                    1,
                    "glider_id",
                    "Could not find glider with name \"Advance Alpha\" in your list of gliders"
                ),
                Message::for_field(
                    1,
                    "launch_at",
                    "Could not find launch site with name \"Züri\" in your list of locations"
                ),
                Message::for_field(
                    1,
                    "landing_at",
                    "Could not find landing site with name \"Rappi\" in your list of locations"
                ),
            ]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].glider_id, None);
        assert_eq!(result.flights[0].launch_at, None);
        assert_eq!(result.flights[0].landing_at, None);
    }

    #[test]
    fn analyze_csv_partial_date_time() {
        let result = analyze("number,date\n42,2023-12-12", None);
        assert_eq!(
            result.warnings,
            vec![
                Message::for_row(
                    1,
                    "If you specify date, launch time or landing time, then the other two values must be provided as well",
                ),
            ]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].launch_at, None);
        assert_eq!(result.flights[0].landing_at, None);
    }

    #[test]
    fn analyze_csv_invalid_date_time() {
        let result = analyze(
            "number,date,launch_time_utc,landing_time_utc\n42,2023-13-44,asdf,2:15 pm",
            None,
        );
        assert_eq!(
            result.warnings,
            vec![
                Message::for_row(1, "Invalid ISO date: 2023-13-44"),
                Message::for_field(1, "launch_time", "Invalid launch time: asdf"),
                Message::for_field(1, "landing_time", "Invalid landing time: 2:15 pm"),
            ]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].launch_time, None);
        assert_eq!(result.flights[0].landing_time, None);
    }

    #[test]
    fn analyze_csv_valid_date_time() {
        let result = analyze(
            "number,date,launch_time_utc,landing_time_utc\n42,2020-03-15,11:13:00,11:18:30",
            None,
        );
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(
            result.flights[0].launch_time,
            Some(DateTime::from_timestamp(1584270780, 0).unwrap())
        );
        assert_eq!(
            result.flights[0].landing_time,
            Some(DateTime::from_timestamp(1584271110, 0).unwrap())
        );
    }

    #[test]
    fn analyze_csv_invalid_xcontest_tracktype_url() {
        let result = analyze(
            "number,xcontest_tracktype,xcontest_url\n42,awesome_flight,xcontest.org/some/flight",
            None,
        );
        assert_eq!(
            result.warnings,
            vec![
                Message::for_field(
                    1,
                    "xcontest_tracktype",
                    "Invalid XContest tracktype: awesome_flight"
                ),
                Message::for_field(
                    1,
                    "xcontest_url",
                    "XContest URL must start with https:// or http://"
                ),
            ]
        );
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(result.flights[0].xcontest_tracktype, None);
        assert_eq!(result.flights[0].xcontest_url, None);
    }

    #[test]
    fn analyze_csv_map_xcontest_url_http() {
        let result = analyze("number,xcontest_url\n42,http://xcontest.org/some/flight", None);
        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());
        assert_eq!(result.flights[0].number, Some(42));
        assert_eq!(
            result.flights[0].xcontest_url,
            Some("https://xcontest.org/some/flight".into())
        );
    }

    #[test]
    fn analyze_csv_full_example() {
        let ctx = DbTestContext::new();

        let glider = data::create_glider(
            &mut ctx.force_get_conn(),
            NewGlider {
                user_id: ctx.testuser1.user.id,
                manufacturer: "Advance".into(),
                model: "Alpha".into(),
                since: None,
                until: None,
                source: None,
                cost: None,
                comment: None,
            },
        )
        .unwrap();
        let züri = data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Züri".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );
        let rappi = data::create_location(
            &mut ctx.force_get_conn(),
            NewLocation {
                name: "Rappi".into(),
                country: "CH".into(),
                elevation: 0,
                user_id: ctx.testuser1.user.id,
                geog: None,
            },
        );

        let result = analyze(
            "number,date,glider,launch_site,launch_time_utc,landing_site,landing_time_utc,track_distance,hikeandfly,comment,xcontest_url,xcontest_tracktype,xcontest_scored_distance,video_url\n\
             1,2020-01-01,Advance Alpha,Züri,10:00:00,Rappi,11:00:00,44.7,,\"Some flying, some scratching\",https://xcontest.org/myflight/,free_flight,27,https://youtube.com/myvid\n\
             2,2020-01-02,Advance Alpha,Rappi,12:01:02,Züri,14:50:50,50,true,Way back,,,,\n\
             ,,,,,,,,false,,,,,\n",
             Some(ctx));

        assert_eq!(result.warnings, empty_vec());
        assert_eq!(result.errors, empty_vec());

        assert_eq!(
            result.flights[0],
            ApiCsvFlightPreview {
                csv_row: 1,
                number: Some(1),
                glider_id: Some(glider.id),
                launch_at: Some(züri.id),
                landing_at: Some(rappi.id),
                launch_time: Some(utc_datetime(2020, 1, 1, 10, 0, 0)),
                landing_time: Some(utc_datetime(2020, 1, 1, 11, 0, 0)),
                track_distance: Some(44.7),
                xcontest_tracktype: Some("free_flight".into()),
                xcontest_distance: Some(27.0),
                xcontest_url: Some("https://xcontest.org/myflight/".into()),
                comment: Some("Some flying, some scratching".into()),
                video_url: Some("https://youtube.com/myvid".into()),
                hikeandfly: false
            }
        );
        assert_eq!(
            result.flights[1],
            ApiCsvFlightPreview {
                csv_row: 2,
                number: Some(2),
                glider_id: Some(glider.id),
                launch_at: Some(rappi.id),
                landing_at: Some(züri.id),
                launch_time: Some(utc_datetime(2020, 1, 2, 12, 1, 2)),
                landing_time: Some(utc_datetime(2020, 1, 2, 14, 50, 50)),
                track_distance: Some(50.0),
                comment: Some("Way back".into()),
                hikeandfly: true,
                ..Default::default()
            }
        );
        assert_eq!(
            result.flights[2],
            ApiCsvFlightPreview {
                csv_row: 3,
                hikeandfly: false,
                ..Default::default()
            }
        );
    }
}
