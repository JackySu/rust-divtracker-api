pub mod user;
pub type DBResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;