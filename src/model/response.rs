use rocket::serde::{Serialize, Deserialize, json::Value};

use std::fmt::{Display, Formatter, Result};
use crate::util::message;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub status: String,
    pub message: String,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ResponseWithStatus {
    pub status_code: u16,
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum ApiStatus {
    Ok,
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
}

impl ApiStatus {
    pub fn code(&self) -> u16 {
        match self {
            ApiStatus::Ok => 200,
            ApiStatus::Created => 201,
            ApiStatus::BadRequest => 400,
            ApiStatus::Unauthorized => 401,
            ApiStatus::Forbidden => 403,
            ApiStatus::NotFound => 404,
            ApiStatus::InternalServerError => 500,
        }
    }
}

impl Display for ApiStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ApiStatus::Ok => write!(f, "{}", message::STATUS_OK),
            ApiStatus::Created => write!(f, "{}", message::STATUS_CREATED),
            ApiStatus::BadRequest => write!(f, "{}", message::STATUS_BAD_REQUEST),
            ApiStatus::Unauthorized => write!(f, "{}", message::STATUS_UNAUTHORIZED),
            ApiStatus::Forbidden => write!(f, "{}", message::STATUS_FORBIDDEN),
            ApiStatus::NotFound => write!(f, "{}", message::STATUS_NOT_FOUND),
            ApiStatus::InternalServerError => write!(f, "{}", message::STATUS_INTERNAL_SERVER_ERROR),
        }
    }
}

impl ResponseWithStatus {
    pub fn new(status: ApiStatus, message: String, data: Option<Value>) -> Self {
        ResponseWithStatus {
            status_code: status.code(),
            response: Response {
                status: status.to_string(),
                message,
                data,
            }
        }
    }
}
