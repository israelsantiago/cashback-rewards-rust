# Porting notes: Spring/Java -> Rust

| Spring/Java | Rust | Porting decision |
|---|---|---|
| `@RestController` | Axum handlers | DTO parsing and HTTP mapping remain at the edge. |
| `@Service` | application service struct | Dependencies are explicit constructor arguments. |
| Inbound interfaces | application port traits | Keep use cases independent from HTTP. |
| Spring Data JPA | SQLx repositories | SQL is explicit; domain remains persistence-ignorant. |
| Flyway | `sqlx::migrate!` | Migrations stay versioned SQL files and run at startup. |
| `BigDecimal` | `rust_decimal::Decimal` | Fixed-point financial arithmetic. |
| JUnit/AssertJ | `cargo test` assertions | Domain and application tests stay fast and framework-light. |
| MockMvc acceptance tests | Axum + service-level acceptance tests | Same black-box HTTP semantics can be tested via Tower/Hyper or a running app. |
| `application.yaml` | environment variables | Twelve-factor configuration style. |

## API numeric representation

The Java API exposes `BigDecimal` response values as JSON numbers. The Rust API therefore serializes response `Decimal` values with `rust_decimal::serde::arbitrary_precision`, preserving exact decimal text and scale (for example `1.60`) without converting financial values through `f64`.

## Known compatibility difference

The Java source exposes runtime exceptions without a dedicated global exception handler. The Rust version makes two business cases explicit:

- duplicate merchant registration -> `409 Conflict`;
- missing fallback rate for an unmapped MCC -> `422 Unprocessable Entity`.

These are intentionally explicit Rust error mappings. They are outside the Java controller contract and are documented here so that the core cashback rules remain separate from adapter-level error policy.

### SQLx migration filenames

SQLx 0.9 expects migration filenames in the form `<VERSION>_<DESCRIPTION>.sql`, where `VERSION` is a positive integer. The Rust port therefore uses `01_...sql`, `02_...sql`, and `03_...sql` instead of the original Flyway `V1__...sql` convention.
