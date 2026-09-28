use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};

pub const POSTGRES_PORT: u16 = 5432;
pub const POSTGRES_DB: &str = "cashback_rewards_test";
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_PASSWORD: &str = "postgres";

pub struct PostgresContext {
    pub pool: PgPool,
    // Keeping the container alive for the complete test scope is essential.
    _container: ContainerAsync<GenericImage>,
}

pub async fn postgres_context() -> Result<PostgresContext, Box<dyn std::error::Error>> {
    let container = GenericImage::new("postgres", "18-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(ContainerPort::Tcp(POSTGRES_PORT))
        .with_env_var("POSTGRES_DB", POSTGRES_DB)
        .with_env_var("POSTGRES_USER", POSTGRES_USER)
        .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container
        .get_host_port_ipv4(ContainerPort::Tcp(POSTGRES_PORT))
        .await?;

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        POSTGRES_USER, POSTGRES_PASSWORD, host, port, POSTGRES_DB
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(PostgresContext {
        pool,
        _container: container,
    })
}
