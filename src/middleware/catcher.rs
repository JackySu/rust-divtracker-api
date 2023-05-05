use rocket::{http::Status, serde::json::Json, response::status};
use crate::model::response::{ApiStatus, Response, ResponseWithStatus};
use crate::util::message;


#[catch(404)]
pub fn not_found() -> status::Custom<Json<Response>> {
    let r = ResponseWithStatus::new(ApiStatus::NotFound, message::MESSAGE_USER_NOT_FOUND.to_string(), None);
    status::Custom(
        Status::from_code(r.status_code).unwrap(),
        Json(r.response),
    )
}

#[catch(429)]
pub fn exceed_rate_limit() -> status::Custom<Json<Response>> {
    let r = ResponseWithStatus::new(ApiStatus::TooManyRequests, message::MESSAGE_TOO_MANY_REQUESTS.to_string(), None);
    status::Custom(
        Status::from_code(r.status_code).unwrap(),
        Json(r.response),
    )
}

#[catch(500)]
pub fn internal_server_error() -> status::Custom<Json<Response>> {
    let r = ResponseWithStatus::new(ApiStatus::InternalServerError, message::MESSAGE_INTERNAL_SERVER_ERROR.to_string(), None);
    status::Custom(
        Status::from_code(r.status_code).unwrap(),
        Json(r.response),
    )
}