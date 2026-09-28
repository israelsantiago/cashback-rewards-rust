# Java -> Rust test correspondence

The Rust test tree mirrors the Java test tree by responsibility:

- `domain/model` -> `tests/domain/model`
- `domain/service` -> `tests/domain/service`
- `application/service` -> `tests/application/service`
- `adapter/in/web` -> `tests/adapter/in/web`
- `adapter/out/persistence` -> `tests/adapter/out/persistence`
- `acceptance` -> `tests/acceptance`

Java repository: `serenity-dojo/cashback-rewards`, branch `section-13/solution`.

| Java test class | Rust counterpart |
|---|---|
| MerchantTest | `tests/domain/model/merchant_test.rs` |
| MinimumPurchaseThresholdTest | `tests/domain/model/minimum_purchase_threshold_test.rs` |
| ProductCategoryTest | `tests/domain/model/product_category_test.rs` |
| CashbackCalculatorTest | `tests/domain/service/cashback_calculator_test.rs` |
| RegisterMerchantServiceTest | `tests/application/service/application_service_tests.rs` |
| ManageProductCategoriesServiceTest | `tests/application/service/application_service_tests.rs` |
| RecordPurchaseServiceTest | `tests/application/service/application_service_tests.rs` |
| ListCustomerCashbackServiceTest | `tests/application/service/application_service_tests.rs` |
| TotalProductCashbackServiceTest | `tests/application/service/application_service_tests.rs` |
| CashbackControllerTest | `tests/adapter/in/web/controller_tests.rs` |
| CategoryControllerTest | `tests/adapter/in/web/controller_tests.rs` |
| MerchantControllerTest | `tests/adapter/in/web/controller_tests.rs` |
| ProductCashbackControllerTest | `tests/adapter/in/web/controller_tests.rs` |
| PurchaseControllerTest | `tests/adapter/in/web/controller_tests.rs` |
| JpaCashbackRepositoryTest | `tests/adapter/out/persistence/repository_integration_tests.rs` |
| JpaCategoryRepositoryTest | `tests/adapter/out/persistence/repository_integration_tests.rs` |
| JpaMerchantRepositoryTest | `tests/adapter/out/persistence/repository_integration_tests.rs` |
| BasicCashbackCalculationIT | `tests/acceptance/postgres_acceptance_tests.rs` + `tests/acceptance/java_gap_scenarios.rs` |
| MerchantCategoriesAndEligibilityIT | `tests/acceptance/postgres_acceptance_tests.rs` + `tests/acceptance/java_gap_scenarios.rs` |
| MinimumPurchaseThresholdIT | `tests/acceptance/postgres_acceptance_tests.rs` + `tests/acceptance/java_gap_scenarios.rs` |
| TotalCashbackPerProductIT | `tests/acceptance/postgres_acceptance_tests.rs` + `tests/acceptance/java_gap_scenarios.rs` |
| CashbackRewardsApplicationTests | `tests/cashback_rewards_application_tests.rs` |

Additional Rust-only acceptance tests are retained; they are not removed merely to force artificial 1:1 file naming.

## Production-like acceptance path

The Rust acceptance flow exercises the same application composition used by `main.rs`: Axum -> inbound ports -> application services -> outbound ports -> `Pg*Repository` -> PostgreSQL. The test-managed PostgreSQL instance is the infrastructure substitute; repositories are not replaced by in-memory doubles.

- `tests/acceptance/api_acceptance_tests.rs` -> HTTP acceptance with production composition and real PostgreSQL
- `tests/acceptance/postgres_acceptance_tests.rs` -> additional production-composition PostgreSQL acceptance scenarios
- `tests/acceptance/java_gap_scenarios.rs` -> Java acceptance-gap scenarios using the same composition
- `tests/adapter/in/web/controller_tests.rs` remains focused on the inbound adapter and may use mocked inbound ports, matching the Java `@WebMvcTest` intent.

