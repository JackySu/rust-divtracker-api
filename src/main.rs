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
use middleware::{governor::RateLimitGuard, cors::Cors, catcher::{exceed_rate_limit, not_found, internal_server_error}};
use api::wrapper::{get_div1_player_stats, get_div2_player_stats};
use api::ubi::login_ubi;

use sqlx::{Pool, Sqlite, SqlitePool};
use anyhow::Result;

#[get("/")]
async fn index() -> &'static str {
    r#"
    - 使用: 当前网址后加上以下路径
        /api/div1/<name> 获取全境1数据（育碧官方api）
        /api/div2/<name> 获取全境2数据（api.tracker.gg）
    - Powered by iulx0 @ 2023
    "#
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
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let pool = SqlitePool::connect(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set").as_str())
        .await
        .expect("Couldn't connect to sqlite database");

    let _ = login_ubi().await?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Couldn't migrate the database tables");

    let config = rocket::Config {
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        ..Default::default()
    };

    let _rocket = rocket::custom(config)
        .mount(
            "/api", 
            routes![get_div1_player_stats_by_name, get_div2_player_stats_by_name])
        .mount("/", routes![index])
        .register(
            "/", 
            catchers![not_found, exceed_rate_limit, internal_server_error])
        .manage(pool)
        .attach(Cors)
        .launch()
        .await?;
    Ok(())
}