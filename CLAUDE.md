# Cashback Rewards — Rust working agreements

## Build

- `cargo test`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run`

## Architecture

- Domain must not import Axum, SQLx or Tokio.
- Application services orchestrate ports; they do not contain HTTP concerns.
- Web adapters map transport DTOs to domain/application types.
- Persistence adapters own SQL and database concerns.

## Money

- Use `rust_decimal::Decimal` for all monetary/rate calculations.
- Never use `f32`/`f64` for money.
- Cashback rounds with `MidpointNearestEven` to scale 2.
- Aggregated totals round down to scale 2, matching the source behavior.

## Development process

Follow:

1. Discover rules/examples/questions from `docs/` and the upstream project.
2. Add or update acceptance coverage.
3. Implement a small TDD cycle.
4. Refactor and review architecture.

Keep tests as executable specifications and avoid duplicating expected calculations inside test assertions.
