#[cfg(feature = "postgres-acceptance")]
#[path = "support/mod.rs"]
pub mod support;

#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/java_gap_scenarios.rs"]
mod java_gap_scenarios;
