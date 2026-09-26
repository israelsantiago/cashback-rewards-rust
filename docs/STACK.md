# Rust stack evaluation

Current package versions were checked against crates.io/docs.rs in September 2026. SQLx 0.9.0 declares Rust 1.94.0 as its rust-version/MSRV for this release cycle.

Evaluated against the current Rust ecosystem available in September 2026.

| Concern | Chosen | Alternatives considered | Why chosen here |
|---|---|---|---|
| Web framework | Axum 0.8 | Actix Web 4, Rocket 0.5 | Thin routing layer, strong Tokio integration, composable middleware, good fit for ports/adapters. |
| Runtime | Tokio 1.x | async-std | Dominant async ecosystem and native fit with Axum/SQLx. |
| DB access | SQLx 0.9 | SeaORM 2 | Keeps SQL explicit and parameterized, async, PostgreSQL-first, with migrations and pooling in one stack. |
| Money | rust_decimal 1.43 | bigdecimal | Finance-friendly fixed precision and direct SQLx NUMERIC support. |
| JSON | Serde | serde_json only | De/serialization derives are idiomatic and pervasive in Rust. |
| Errors | thiserror 2 | anyhow | Typed errors are preferable in domain/application boundaries. |
| Observability | tracing + tower-http | log/env_logger | Structured spans/events and HTTP request tracing. |
| OpenAPI | utoipa 6 + Swagger UI 10 | hand-authored OpenAPI | Keeps the API contract close to DTOs and handlers. |
| Integration DB tests | PostgreSQL service in CI | testcontainers-rs | Fewer moving pieces for this small project; Testcontainers remains a good next step when per-test isolation is required. |

### Why not Actix Web?

Actix Web remains a mature and well-supported option. The main reason for not using it here is architectural consistency with a Tokio-first stack: Axum, Tokio, SQLx and tower middleware compose naturally without adding a second framework style.

### Why not SeaORM?

SeaORM is a legitimate choice when an application benefits from higher-level entity/relation abstractions. This project is intentionally small and already has a stable relational schema, so direct SQL provides a smaller conceptual gap between business requirements and persistence.

### Why rust_decimal instead of f64?

Cashback is a financial calculation. Binary floating point introduces representation/rounding concerns; `rust_decimal` gives fixed-precision decimal arithmetic and can map PostgreSQL `NUMERIC` values through SQLx.
