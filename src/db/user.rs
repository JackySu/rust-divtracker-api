use super::DBResult;
use rocket::futures::{stream::TryStreamExt, TryFutureExt};
use sqlx::{Pool, Sqlite};

pub async fn get_user_names_by_id(pool: &Pool<Sqlite>, id: &str) -> DBResult<Vec<String>> {
    let mut connection = pool.acquire().await?;
    let names = sqlx::query!(
        r#"
        SELECT name FROM user_names WHERE user_id = $1 ORDER BY ts DESC;
        "#,
        id
    )
    .fetch(&mut *connection)
    .map_ok(|r| r.name)
    .try_collect::<Vec<_>>().await?;

    Ok(names)
}

pub async fn get_user_id_by_name(pool: &Pool<Sqlite>, name: &str) -> DBResult<Vec<String>> {
    let mut connection = pool.acquire().await?;
    let id = sqlx::query!(
        r#"
        SELECT user_id FROM user_names WHERE LOWER(name) = LOWER($1) ORDER BY ts DESC;
        "#,
        name
    )
    .fetch(&mut *connection)
    .map_ok(|r| r.user_id)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(id)
}

pub async fn create_user(pool: &Pool<Sqlite>, id: &str) -> DBResult<bool> {
    let mut connection = pool.acquire().await?;
    let r = sqlx::query!(
        r#"
        INSERT INTO user_ids (id) VALUES ($1) ON CONFLICT DO NOTHING;
        "#,
        id
    )
    .execute(&mut *connection)
    .await?
    .rows_affected();

    Ok(r > 0)
}

pub async fn store_user_name(pool: &Pool<Sqlite>, id: &str, name: &str) -> DBResult<bool> {
    let mut connection = pool.acquire().await?;
    let r = sqlx::query!(
        r#"
        INSERT INTO user_names (user_id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING;
        "#,
        id,
        name
    )
    .execute(&mut *connection)
    .await?
    .rows_affected();

    Ok(r > 0)
}