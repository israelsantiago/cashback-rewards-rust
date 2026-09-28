# Cashback Rewards Rust — Hexagonal Architecture / Ports & Adapters

## Baseline

The Rust implementation is aligned conceptually 1:1 with the Java `section-13/solution` baseline of Serenity Dojo's `cashback-rewards`.

The Rust names are adapted to Rust conventions (`inbound` / `outbound` instead of Java `in` / `out`) but the architectural responsibilities are preserved.

## 1. Java → Rust mapping

| Java | Rust | Responsibility |
|---|---|---|
| `domain/model/Merchant` | `domain/model/merchant.rs` | Business model |
| `domain/model/ProductCategory` | `domain/model/product_category.rs` | Business model + `Other` fallback |
| `domain/model/CashbackRecord` | `domain/model/cashback_record.rs` | Business model |
| `domain/model/ProductCashbackTotal` | `domain/model/product_cashback_total.rs` | Aggregate result |
| `domain/model/MinimumPurchaseThreshold` | `domain/model/minimum_purchase_threshold.rs` | Minimum purchase rule |
| `domain/service/CashbackCalculator` | `domain/service/cashback_calculator.rs` | Cashback calculation |
| `domain/exception/*` | `domain/error.rs` | Domain errors |
| `application/port/in/*` | `application/port/in/*` | Inbound ports / use cases |
| `application/port/out/*` | `application/port/out/*` | Outbound ports |
| `application/service/*` | `application/service/*` | Use-case implementations |
| `adapter/in/web/*Controller` | `adapter/in/web/*_controller.rs` | HTTP inbound adapters |
| `adapter/out/persistence/*Repository` | `adapter/out/persistence/*_repository.rs` | PostgreSQL outbound adapters |
| JPA entities | persistence `*_entity.rs` | Persistence representation, kept outside the domain |
| Spring composition | `main.rs` | Composition root |

## 2. Dependency rule

```text
                +----------------------------+
                |      INBOUND ADAPTERS      |
                |       Axum / HTTP          |
                +-------------+--------------+
                              |
                              | depends on
                              v
                +----------------------------+
                |       INBOUND PORTS        |
                |       application/         |
                |       port/inbound          |
                +-------------+--------------+
                              |
                              v
                +----------------------------+
                |     APPLICATION SERVICES   |
                |       use-case logic        |
                +-------------+--------------+
                              |
                    depends on abstractions
                              |
                              v
                +----------------------------+
                |      OUTBOUND PORTS        |
                | application/port/out   |
                +-------------+--------------+
                              ^
                              |
                    implements the ports
                              |
                +-------------+--------------+
                |    OUTBOUND ADAPTERS        |
                | SQLx / PostgreSQL           |
                +----------------------------+

                +----------------------------+
                |          DOMAIN             |
                | models + business rules    |
                +----------------------------+
```

The key rule is that the web adapter does **not** depend on concrete application services, and application services do **not** depend on PostgreSQL implementations.

## 3. Inbound side

The Java controllers depend on `application.port.in.*`. The Rust controllers now depend on `application.port.inbound::*` trait objects held by `WebState`.

```text
PurchaseController
       |
       v
RecordPurchaseUseCase
       |
       v
RecordPurchaseService
```

The same pattern applies to merchant registration, category management, customer cashback listing and product cashback totals.

`WebState` is intentionally located in the inbound web adapter. It is not an application-layer service locator.

## 4. Outbound side

The Java application services depend on repository interfaces from `application.port.out.*`. Rust does the same with `application.port.outbound::*`.

```text
RecordPurchaseService
       |
       +----> MerchantRepository
       |
       +----> CategoryRepository
       |
       +----> CashbackRepository

                ^
                |
       PostgreSQL adapters
```

Each production adapter is isolated in its own module:

```text
adapter/out/persistence/
├── merchant_repository.rs
├── category_repository.rs
└── cashback_repository.rs
```

## 5. Persistence model isolation

The Rust SQLx rows are deliberately separated from the domain models:

```text
PostgreSQL
    |
    v
MerchantEntity / ProductCategoryEntity /
CashbackRecordEntity / DefaultCashbackRateEntity
    |
    | mapping
    v
Domain model
```

This mirrors the Java JPA adapter, where `MerchantEntity`, `ProductCategoryEntity`, `CashbackRecordEntity` and `DefaultCashbackRateEntity` are infrastructure representations.

## 6. Composition root

`main.rs` is the only place where concrete adapters and application services are assembled:

```text
PgMerchantRepository
PgCategoryRepository
PgCashbackRepository
          |
          v
Application Services
          |
          v
WebState (inbound ports only)
          |
          v
Axum Router
```

This is the Rust equivalent of the Spring dependency-injection composition performed by `@Service`, `@Repository` and constructor injection.

## 7. RecordPurchase contract

The Java inbound port receives:

```java
record(String customerId,
       String merchantName,
       String mcc,
       BigDecimal amount,
       Instant purchasedAt)
```

The Rust port now preserves the same contract with `DateTime<Utc>` for the `Instant` equivalent. The timestamp is deliberately not persisted because the Java application service also does not persist it.

## 8. Persistence schema

The Rust migration filenames use numeric prefixes because SQLx migration naming differs from Flyway:

```text
Java / Flyway                 Rust / SQLx
V1__create_merchant_table     01_create_merchant_table.sql
V2__create_product_category   02_create_product_category_table.sql
V3__create_cashback_record    03_create_cashback_record_table.sql
```

The table definitions remain aligned with the Java baseline. The Rust-only product-category index was removed to keep the schema equivalent. Category persistence also uses an `ON CONFLICT (mcc) DO UPDATE` upsert because the Java `JpaRepository.save()` persists an existing category by its identifier rather than rejecting it.

## 9. Tests

The test strategy mirrors the Java solution while separating concerns more strictly:

```text
Domain unit tests
    -> #[cfg(test)] inside domain modules; no infrastructure

Application service unit tests
    -> small port stubs; no database implementation is re-created

Persistence adapter integration tests
    -> real PostgreSQL in Testcontainers

HTTP acceptance tests
    -> Axum + real PostgreSQL adapters + Testcontainers
```

The Java solution contains four domain unit-test classes (`MerchantTest`, `MinimumPurchaseThresholdTest`, `ProductCategoryTest`, `CashbackCalculatorTest`), application-service unit tests using in-memory repository doubles, and `@DataJpaTest` repository tests. The Rust port now represents those concerns explicitly instead of treating an in-memory repository as a substitute for PostgreSQL integration testing.

Testcontainers is therefore not a replacement for unit tests: it verifies the outbound adapter and database contract. The small Rust port stubs used by application-service tests are deliberately behavior-oriented and do not attempt to emulate SQL, indexing, normalization persistence, aggregation queries, or PostgreSQL semantics. Those behaviors are tested against the actual PostgreSQL adapters.

## 10. Wire-contract and adapter parity checks

The following details were explicitly checked against the Java baseline:

- Web controllers receive only inbound-port abstractions; no controller stores or references a concrete application service.
- Outbound repository ports are implemented by separate PostgreSQL adapter types.
- SQLx persistence entities are distinct from domain models, matching the JPA entity/domain separation.
- `RecordPurchaseUseCase` preserves the Java `purchasedAt` argument even though the current Java service does not store it.
- `CategoryRepository.save` has update-on-existing-MCC semantics equivalent to JPA `save`.
- The extra Rust `/health` endpoint was removed from the application router because it has no corresponding Java controller endpoint.
- Decimal response fields are serialized as JSON numbers, not JSON strings, to match Jackson `BigDecimal` output.

## 11. Intentional Rust-specific differences

These are implementation-language differences, not architectural differences:

- Java interfaces become Rust traits.
- Spring dependency injection becomes explicit composition in `main.rs`.
- JPA entities become SQLx row/entity structs.
- Java `Instant` becomes `chrono::DateTime<Utc>`.
- Java synchronous ports/services are asynchronous Rust traits because SQLx I/O is asynchronous.
- Java `in` / `out` packages are named `inbound` / `outbound` to avoid keyword-related awkwardness in Rust.
