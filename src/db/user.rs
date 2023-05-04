use super::DBResult;
use rocket::futures::{stream::TryStreamExt, future::TryFutureExt};
use sqlx::{Pool, Sqlite};

pub async fn get_user_names_by_id(pool: &Pool<Sqlite>, id: &str) -> DBResult<Vec<String>> {
    let mut connection = pool.acquire().await?;
    let names = sqlx::query!(
        r#"
        SELECT name FROM user_names WHERE user_id = ? ORDER BY ts DESC;
        "#,
        id
    )
    .fetch(&mut *connection)
    .map_ok(|r| r.name)
    .try_collect::<Vec<_>>().await?;

    Ok(names)
}

pub async fn create_user(pool: &Pool<Sqlite>, id: &str) -> DBResult<bool> {
    let mut connection = pool.acquire().await?;
    let r = sqlx::query!(
        r#"
        INSERT INTO user_ids (id) VALUES (?) ON CONFLICT DO NOTHING;
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
        INSERT INTO user_names (user_id, name) VALUES (?, ?) ON CONFLICT DO NOTHING;
        "#,
        id,
        name
    )
    .execute(&mut *connection)
    .await?
    .rows_affected();

    Ok(r > 0)
}