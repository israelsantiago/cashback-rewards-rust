#![allow(dead_code)]

use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::{Duration, Instant, sleep};

const POSTGRES_PORT: u16 = 5432;
const POSTGRES_DB: &str = "cashback";
const POSTGRES_USER: &str = "cashback";
const POSTGRES_PASSWORD: &str = "cashback";

pub struct PgTestContext {
    pub _container: ContainerAsync<GenericImage>,
    pub pool: PgPool,
}

pub async fn postgres_context() -> Result<PgTestContext, Box<dyn std::error::Error>> {
    let container = GenericImage::new("postgres", "18-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(POSTGRES_PORT.tcp())
        .with_env_var("POSTGRES_DB", POSTGRES_DB)
        .with_env_var("POSTGRES_USER", POSTGRES_USER)
        .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(POSTGRES_PORT.tcp()).await?;
    let url = format!("postgres://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{host}:{port}/{POSTGRES_DB}");

    let deadline = Instant::now() + Duration::from_secs(30);
    let pool = loop {
        match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => break pool,
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                sleep(Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    };

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(PgTestContext {
        _container: container,
        pool,
    })
}
