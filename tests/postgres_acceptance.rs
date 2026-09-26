#![cfg(feature = "postgres-acceptance")]

use std::env;

use sqlx::{postgres::PgPoolOptions, PgPool};

async fn pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url = env::var("DATABASE_URL")?;
    Ok(PgPoolOptions::new().max_connections(5).connect(&url).await?)
}

#[tokio::test]
async fn migrations_apply() -> Result<(), Box<dyn std::error::Error>> {
    let pool = pool().await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let exists: bool = sqlx::query_scalar(
        "SELECT to_regclass('public.cashback_record') IS NOT NULL",
    )
    .fetch_one(&pool)
    .await?;

    assert!(exists);
    Ok(())
}
