# Java → Rust — Matriz de Rastreabilidade da Suíte de Testes

Fonte Java: `serenity-dojo/cashback-rewards`, branch `section-13/solution`.

Fonte: <https://github.com/serenity-dojo/cashback-rewards/tree/section-13/solution/src/test/java/com/serenitydojo/cashback_rewards>

| # | Java | Rust | Forma de adaptação |
|---:|---|---|---|
| 1 | `CashbackRewardsApplicationTests` | `tests/application_context_test.rs` | Smoke/context nativo Rust |
| 2 | `MerchantTest` | `tests/domain/model/merchant_test.rs` | Unit test |
| 3 | `MinimumPurchaseThresholdTest` | `tests/domain/model/minimum_purchase_threshold_test.rs` | Unit test |
| 4 | `ProductCategoryTest` | `tests/domain/model/product_category_test.rs` | Unit test |
| 5 | `CashbackCalculatorTest` | `tests/domain/service/cashback_calculator_test.rs` | Unit test |
| 6 | `ListCustomerCashbackServiceTest` | `tests/application/service/application_service_tests.rs` | Teste de use case com double |
| 7 | `ManageProductCategoriesServiceTest` | `tests/application/service/application_service_tests.rs` | Teste de use case com double |
| 8 | `RecordPurchaseServiceTest` | `tests/application/service/application_service_tests.rs` | Teste de use case com doubles |
| 9 | `RegisterMerchantServiceTest` | `tests/application/service/application_service_tests.rs` | Teste de use case + duplicate/normalization |
| 10 | `TotalProductCashbackServiceTest` | `tests/application/service/application_service_tests.rs` | Total/count/scale 2 |
| 11 | `CashbackControllerTest` | `tests/adapter/in/web/controller_tests.rs` | Axum Router + `oneshot` |
| 12 | `CategoryControllerTest` | `tests/adapter/in/web/controller_tests.rs` | Axum + JSON |
| 13 | `MerchantControllerTest` | `tests/adapter/in/web/controller_tests.rs` | Axum + JSON |
| 14 | `ProductCashbackControllerTest` | `tests/adapter/in/web/controller_tests.rs` | Axum + JSON |
| 15 | `PurchaseControllerTest` | `tests/adapter/in/web/controller_tests.rs` | Axum + JSON |
| 16 | `JpaCashbackRepositoryTest` | `tests/adapter/out/persistence/repository_integration_tests.rs` | PostgreSQL + Testcontainers |
| 17 | `JpaCategoryRepositoryTest` | `tests/adapter/out/persistence/repository_integration_tests.rs` | PostgreSQL + Testcontainers |
| 18 | `JpaMerchantRepositoryTest` | `tests/adapter/out/persistence/repository_integration_tests.rs` | PostgreSQL + Testcontainers |
| 19 | `BasicCashbackCalculationIT` | `tests/acceptance/postgres_acceptance_tests.rs` | E2E/integration com PostgreSQL |
| 20 | `MerchantCategoriesAndEligibilityIT` | `tests/acceptance/postgres_acceptance_tests.rs` | E2E/integration com PostgreSQL |
| 21 | `MinimumPurchaseThresholdIT` | `tests/acceptance/postgres_acceptance_tests.rs` | E2E/integration com PostgreSQL |
| 22 | `TotalCashbackPerProductIT` | `tests/acceptance/postgres_acceptance_tests.rs` | E2E/integration com PostgreSQL |

## Test doubles preservados

Os três doubles presentes no Java continuam representados na estratégia Rust:

- `InMemoryMerchantRepository`
- `InMemoryCategoryRepository`
- `InMemoryCashbackRepository`

A implementação Rust pode usar structs/traits/stubs em vez de reproduzir classes Java literalmente. O requisito é preservar o isolamento do application service test.

## Cobertura funcional preservada

A matriz precisa continuar cobrindo:

- merchant partner/non-partner;
- merchant duplicate/case/whitespace normalization;
- category by MCC;
- default cashback rate;
- unmapped MCC → `Other`;
- minimum purchase threshold `1.00`;
- cálculo `amount × rate`;
- rounding `HALF_EVEN` em 2 casas;
- listagem por customer;
- total por categoria/produto;
- count de cashback records;
- zero total para categoria sem registros;
- payloads HTTP e campos de request/response;
- persistência PostgreSQL e migrations.

## Critério de conclusão

A matriz é considerada concluída quando:

```text
22/22 cenários/classes com equivalente Rust
+
cargo test --workspace --all-targets --all-features
+
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

A equivalência de responsabilidade importa mais do que a reprodução literal do número de arquivos Java.
