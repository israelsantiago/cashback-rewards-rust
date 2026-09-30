#[cfg(feature = "postgres-acceptance")]
#[path = "support/mod.rs"]
pub mod support;

#[cfg(feature = "postgres-acceptance")]
#[path = "adapter/out/persistence/repository_integration_tests.rs"]
mod repository_integration_tests;
