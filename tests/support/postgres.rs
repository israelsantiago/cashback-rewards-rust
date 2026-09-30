use std::sync::{Arc, OnceLock};
use std::time::Instant;

use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::sync::{Mutex, OnceCell};
use tracing::info;

use crate::support::runtime::run_async;

pub const POSTGRES_PORT: u16 = 5432;
pub const POSTGRES_DB: &str = "cashback_rewards_test";
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_PASSWORD: &str = "postgres";

struct SharedPostgres {
    pool: PgPool,
    // Kept alive for the lifetime of this integration-test process.
    _container: ContainerAsync<GenericImage>,
}

static POSTGRES: OnceCell<Arc<SharedPostgres>> = OnceCell::const_new();
static DEFAULT_RATE_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

async fn initialize_postgres()
-> Result<Arc<SharedPostgres>, Box<dyn std::error::Error + Send + Sync>> {
    let started = Instant::now();
    info!(target: "cashback_rewards_rust::test_timing", event = "postgres.starting");

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

    crate::support::orphan_reaper::track_container(container.id());

    let host = container.get_host().await?;
    let port = container
        .get_host_port_ipv4(ContainerPort::Tcp(POSTGRES_PORT))
        .await?;

    info!(
        target: "cashback_rewards_rust::test_timing",
        event = "postgres.container_ready",
        elapsed_ms = started.elapsed().as_millis() as u64,
        %host,
        port,
    );

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        POSTGRES_USER, POSTGRES_PASSWORD, host, port, POSTGRES_DB
    );

    let connect_started = Instant::now();
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .connect(&database_url)
        .await?;
    info!(
        target: "cashback_rewards_rust::test_timing",
        event = "postgres.pool_ready",
        elapsed_ms = connect_started.elapsed().as_millis() as u64,
    );

    let migrate_started = Instant::now();
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!(
        target: "cashback_rewards_rust::test_timing",
        event = "postgres.migrations_completed",
        elapsed_ms = migrate_started.elapsed().as_millis() as u64,
    );

    info!(
        target: "cashback_rewards_rust::test_timing",
        event = "postgres.environment_ready",
        elapsed_ms = started.elapsed().as_millis() as u64,
    );

    Ok(Arc::new(SharedPostgres {
        pool,
        _container: container,
    }))
}

pub async fn postgres_context() -> Result<PgPool, Box<dyn std::error::Error>> {
    let shared = POSTGRES
        .get_or_try_init(initialize_postgres)
        .await
        .map_err(|error| -> Box<dyn std::error::Error> { error.to_string().into() })?;
    Ok(shared.pool.clone())
}

/// Runs an async future on the single runtime that owns the shared SQLx pool.
pub fn postgres_test<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    run_async(future)
}

/// Global state that affects unrelated fixtures must be protected explicitly.
/// The current schema has a singleton default cashback rate, so scenarios that
/// modify/read that setting use this guard. Ordinary fixture-scoped data does
/// not need a lock because its identity is unique.
pub async fn default_rate_guard() -> tokio::sync::OwnedMutexGuard<()> {
    DEFAULT_RATE_LOCK
        .get_or_init(|| Arc::new(Mutex::new(())))
        .clone()
        .lock_owned()
        .await
}
