use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response,
    serde::json::{self, Json},
    Request,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RocketError {
    pub error: RocketErrorInner,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RocketErrorInner {
    pub code: u16,
    pub reason: &'static str,
    pub description: &'static str,
}

impl RocketError {
    pub fn new(
        status: Status,
        reason: &'static str,
        description: &'static str,
    ) -> (Status, Json<RocketError>) {
        (
            status,
            Json(RocketError {
                error: RocketErrorInner {
                    code: status.code,
                    reason,
                    description,
                },
            }),
        )
    }
}

pub enum ApiError {
    MissingAuthentication,
}

#[rocket::async_trait]
impl<'r> response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, body) = match self {
            ApiError::MissingAuthentication => (
                Status::Unauthorized,
                json::to_string(&RocketError {
                    error: RocketErrorInner {
                        code: 401,
                        reason: "MissingAuthentication",
                        description: "Not authenticated, please log in",
                    },
                })
                .unwrap(),
            ),
        };
        response::Response::build()
            .header(ContentType::JSON)
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
