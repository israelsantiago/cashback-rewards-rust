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

## Intentional API representation

Money and rates are represented as JSON strings (`"12.40"`, `"0.05"`) rather than JSON floating-point values. This keeps the wire representation stable and avoids accidental IEEE-754 conversion in clients.

## Known compatibility difference

The Java source exposes runtime exceptions without a dedicated global exception handler. The Rust version makes two business cases explicit:

- duplicate merchant registration -> `409 Conflict`;
- missing fallback rate for an unmapped MCC -> `422 Unprocessable Entity`.

These are intentional API-quality improvements, not changes to the core cashback rules.
