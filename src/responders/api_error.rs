use std::{borrow::Cow, io::Cursor};

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
    pub description: Cow<'static, str>,
}

impl RocketError {
    pub fn new(
        status: Status,
        reason: &'static str,
        description: impl Into<Cow<'static, str>>,
    ) -> (Status, Json<RocketError>) {
        (
            status,
            Json(RocketError {
                error: RocketErrorInner {
                    code: status.code,
                    reason,
                    description: description.into(),
                },
            }),
        )
    }
}

pub enum ApiError {
    MissingAuthentication,
    InvalidData { message: String },
    NotFound,
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
                        description: "Not authenticated, please log in".into(),
                    },
                })
                .unwrap(),
            ),
            ApiError::InvalidData { message } => (
                Status::BadRequest,
                json::to_string(&RocketError {
                    error: RocketErrorInner {
                        code: 400,
                        reason: "InvalidData",
                        description: message.into(),
                    },
                })
                .unwrap(),
            ),
            ApiError::NotFound => (
                Status::NotFound,
                json::to_string(&RocketError {
                    error: RocketErrorInner {
                        code: 404,
                        reason: "NotFound",
                        description: "Requested data not found".into(),
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
