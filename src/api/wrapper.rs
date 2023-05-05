use serde_json::json;
use sqlx::{Pool, Sqlite};

use crate::api::ubi;
use crate::util::message;
use crate::model::response::{ApiStatus, ResponseWithStatus};


pub async fn get_div1_player_stats(pool: &Pool<Sqlite>, name: &str) -> ResponseWithStatus {
    let stats = ubi::get_div1_player_stats(pool, name).await;
    match stats {
        Ok(stats) => {
            ResponseWithStatus::new(ApiStatus::Ok, message::MESSAGE_USER_EXISTS.to_string(), Some(json!(stats)))
        },
        Err(err) => {
            println!("Error: {}", err);
            ResponseWithStatus::new(ApiStatus::BadRequest, message::MESSAGE_USER_NOT_FOUND.to_string(), None)
        }
    }
}

pub async fn get_div2_player_stats(pool: &Pool<Sqlite>, name: &str) -> ResponseWithStatus {
    let stats = ubi::get_div2_player_stats(pool, name).await;
    match stats {
        Ok(stats) => {
            ResponseWithStatus::new(ApiStatus::Ok, message::MESSAGE_USER_EXISTS.to_string(), Some(json!(stats)))
        },
        Err(err) => {
            println!("Error: {}", err);
            ResponseWithStatus::new(ApiStatus::BadRequest, message::MESSAGE_USER_NOT_FOUND.to_string(), None)
        }
    }
}