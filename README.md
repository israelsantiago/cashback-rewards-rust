# Cashback Rewards — Rust

Rust conversion of the [Serenity Dojo Cashback Rewards](https://github.com/serenity-dojo/cashback-rewards) hands-on project, targeting the `section-13/solution` branch as the source baseline.

The original sample uses Spring Boot, JPA, Flyway and PostgreSQL. This version preserves the core API and business rules while replacing the Java/Spring stack with a Rust-native stack and keeping the hexagonal architecture.

## Rust stack

- **Axum 0.8** — HTTP routing and handlers.
- **Tokio 1.x** — asynchronous runtime.
- **SQLx 0.9** — PostgreSQL access, connection pool and embedded migrations.
- **rust_decimal** — exact decimal arithmetic for money/rates.
- **Serde** — JSON serialization/deserialization.
- **thiserror** — typed domain/application errors.
- **tracing + tower-http** — structured diagnostics and HTTP tracing.
- **utoipa + Swagger UI** — OpenAPI generated from the Rust API types.

The main design choice is deliberate: SQLx keeps parameterized SQL and migrations explicit instead of introducing an ORM layer, while Axum keeps the web adapter thin. Both fit the original ports-and-adapters structure well.

## Architecture

The package structure mirrors the Java hexagonal baseline while using Rust naming conventions:

```text
src/
├── domain/
│   ├── model/                         # business models
│   ├── service/                       # domain services
│   └── error.rs                       # domain errors
├── application/
│   ├── port/inbound/                  # use-case ports
│   ├── port/outbound/                 # persistence ports
│   └── service/                       # use-case implementations
├── adapter/
│   ├── inbound/web/                   # Axum controllers + DTOs
│   └── outbound/persistence/           # SQLx adapters + persistence entities
└── main.rs                            # composition root
```

Dependencies point toward the application/domain core: HTTP controllers depend only on inbound ports; application services depend only on outbound ports; PostgreSQL adapters implement those outbound ports. See [`docs/CLEAN_ARCHITECTURE.md`](docs/CLEAN_ARCHITECTURE.md) for the Java -> Rust 1:1 mapping.

## Implemented behavior

The Rust application preserves the current production code behavior of the source branch for:

- partner vs. non-partner merchant eligibility;
- configurable MCC/category cashback rates;
- fallback `Other` category using the default rate;
- minimum purchase threshold of **$1.00**;
- `HALF_EVEN` cashback calculation at 2 decimal places;
- customer cashback listing;
- aggregate product-category cashback totals and record counts;
- merchant-name normalization for lookups.

The source repository's OpenAPI/spec documentation also describes a monthly cashback report endpoint, but that endpoint is not implemented by the Java production code in `section-13/solution`; it is therefore documented as follow-up work rather than being invented in this port.

## Run locally

```bash
docker compose up --build
```

API: `http://localhost:8080`

Swagger UI: `http://localhost:8080/swagger-ui`

OpenAPI JSON: `http://localhost:8080/api-docs/openapi.json`


## Configuration

```bash
cp .env.example .env
```

Environment variables:

- `DATABASE_URL`
- `BIND_ADDR`
- `RUST_LOG`

## Test

Unit tests run without PostgreSQL:

```bash
cargo test
```

CI also runs Clippy, HTTP acceptance tests with in-memory ports, and PostgreSQL end-to-end acceptance tests using Testcontainers (`--all-features`).

## Example

```bash
curl -X POST http://localhost:8080/api/categories \
  -H 'content-type: application/json' \
  -d '{"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}'

curl -X POST http://localhost:8080/api/merchants \
  -H 'content-type: application/json' \
  -d '{"name":"GreenGrocer","partner":true}'

curl -X POST http://localhost:8080/api/purchases \
  -H 'content-type: application/json' \
  -d '{"customerId":"cust-001","merchantName":"GreenGrocer","amount":"80.00","mcc":"5411","purchasedAt":"2026-05-01T10:00:00Z"}'

curl http://localhost:8080/api/customers/cust-001/cashback
```

## Licensing

No `LICENSE` file was present in the upstream branch used as the baseline during this conversion. This port therefore does not declare an open-source license. See [`docs/LICENSING.md`](docs/LICENSING.md) before redistribution.

## Source attribution

This project is a technical port of the public Serenity Dojo sample. The original requirements/specifications and learning material remain the reference for business intent. This repository is independently structured for Rust and does not copy the Java implementation line-for-line.
