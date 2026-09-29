use std::{
    future::Future,
    sync::{Once, OnceLock},
};

use tokio::runtime::{Builder, Runtime};
use tracing_subscriber::{EnvFilter, fmt};

static TEST_RUNTIME: OnceLock<Runtime> = OnceLock::new();
static TRACING_INIT: Once = Once::new();

pub fn init_test_tracing() {
    TRACING_INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("cashback_rewards_rust::test_timing=info"));

        let _ = fmt()
            .with_env_filter(filter)
            .with_test_writer()
            .with_ansi(false)
            .compact()
            .try_init();
    });
}

pub fn test_runtime() -> &'static Runtime {
    TEST_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .thread_name("cashback-test-runtime")
            .build()
            .expect("failed to create shared test runtime")
    })
}

pub fn run_async<F>(future: F) -> F::Output
where
    F: Future,
{
    init_test_tracing();
    test_runtime().block_on(future)
}
