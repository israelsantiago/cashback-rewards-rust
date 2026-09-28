#[path = "adapter/in/web/api_acceptance_tests.rs"]
mod api_acceptance_tests;
#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/postgres_acceptance_tests.rs"]
mod postgres_acceptance_tests;
#[cfg(feature = "postgres-acceptance")]
#[path = "adapter/out/persistence/repository_integration_tests.rs"]
mod repository_integration_tests;
