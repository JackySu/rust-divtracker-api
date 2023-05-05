#[macro_use]
extern crate rocket;
extern crate lazy_static;

mod api;
mod model;
mod util;
mod middleware;
mod db;

use rocket::State;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket_governor::RocketGovernor;

use model::response::Response;
use middleware::{governor::RateLimitGuard, catcher::{exceed_rate_limit, not_found, internal_server_error}};
use api::wrapper::{get_div1_player_stats, get_div2_player_stats};
use api::ubi::login_ubi;

use sqlx::{Pool, Sqlite, SqlitePool};

#[get("/")]
async fn index() -> &'static str {
    "GET /div1/<name>\nGET /div2/<name>" 
}

#[get("/div1/<name>")]
async fn get_div1_player_stats_by_name(_limitguard: RocketGovernor<'_, RateLimitGuard>, pool: &State<Pool<Sqlite>>, name: &str) -> status::Custom<Json<Response>> {
    let stats = get_div1_player_stats(pool, name).await;
    status::Custom(
        Status::from_code(stats.status_code).unwrap(),
        Json(stats.response),
    )
}

#[get("/div2/<name>")]
async fn get_div2_player_stats_by_name(_limitguard: RocketGovernor<'_, RateLimitGuard>, pool: &State<Pool<Sqlite>>, name: &str) -> status::Custom<Json<Response>> {
    let stats = get_div2_player_stats(pool, name).await;
    status::Custom(
        Status::from_code(stats.status_code).unwrap(),
        Json(stats.response),
    )
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv::dotenv().ok();
    let pool = SqlitePool::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set").as_str())
        .await
        .expect("Couldn't connect to sqlite database");

    let _ = login_ubi().await;
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Couldn't migrate the database tables");
    
    let _rocket = rocket::build()
        .mount("/", routes![
            index, get_div1_player_stats_by_name, get_div2_player_stats_by_name
        ])
        .register("/", catchers![
            not_found, exceed_rate_limit, internal_server_error
        ])
        .manage(pool)
        .launch()
        .await?;
    Ok(())
}