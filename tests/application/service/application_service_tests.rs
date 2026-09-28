use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, dec};

use cashback_rewards_rust::application::ApplicationError;
use cashback_rewards_rust::application::port::{
    r#in::{
        ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
        RegisterMerchantUseCase, TotalProductCashbackUseCase,
    },
    out::{CashbackRepository, CategoryRepository, MerchantRepository, PortError},
};
use cashback_rewards_rust::application::service::{
    ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
    RegisterMerchantService, TotalProductCashbackService,
};
use cashback_rewards_rust::domain::exception::DomainError;
use cashback_rewards_rust::domain::model::{
    CashbackRecord, Merchant, ProductCashbackTotal, ProductCategory,
};

#[derive(Default)]
struct StubMerchantRepository {
    find_result: Mutex<Option<Merchant>>,
    saved: Mutex<Vec<Merchant>>,
}

#[async_trait]
impl MerchantRepository for StubMerchantRepository {
    async fn save(&self, merchant: &Merchant) -> Result<(), PortError> {
        self.saved.lock().unwrap().push(merchant.clone());
        Ok(())
    }

    async fn find_by_name(&self, _name: &str) -> Result<Option<Merchant>, PortError> {
        Ok(self.find_result.lock().unwrap().clone())
    }
}

#[derive(Default)]
struct StubCategoryRepository {
    find_result: Mutex<Option<ProductCategory>>,
    default_rate: Mutex<Option<Decimal>>,
    saved: Mutex<Vec<ProductCategory>>,
    configured_rates: Mutex<Vec<Decimal>>,
}

#[async_trait]
impl CategoryRepository for StubCategoryRepository {
    async fn save(&self, category: &ProductCategory) -> Result<(), PortError> {
        self.saved.lock().unwrap().push(category.clone());
        Ok(())
    }

    async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError> {
        self.configured_rates.lock().unwrap().push(rate);
        Ok(())
    }

    async fn default_rate(&self) -> Result<Option<Decimal>, PortError> {
        Ok(*self.default_rate.lock().unwrap())
    }

    async fn find_by_mcc(&self, _mcc: &str) -> Result<Option<ProductCategory>, PortError> {
        Ok(self.find_result.lock().unwrap().clone())
    }
}

#[derive(Default)]
struct StubCashbackRepository {
    saved: Mutex<Vec<CashbackRecord>>,
    list_result: Mutex<Vec<CashbackRecord>>,
    total_result: Mutex<Decimal>,
    count_result: Mutex<i64>,
}

#[async_trait]
impl CashbackRepository for StubCashbackRepository {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError> {
        self.saved.lock().unwrap().push(record.clone());
        Ok(())
    }

    async fn find_by_customer_id(
        &self,
        _customer_id: &str,
    ) -> Result<Vec<CashbackRecord>, PortError> {
        Ok(self.list_result.lock().unwrap().clone())
    }

    async fn total_for_product_category(
        &self,
        _product_category: &str,
    ) -> Result<Decimal, PortError> {
        Ok(*self.total_result.lock().unwrap())
    }

    async fn count_for_product_category(&self, _product_category: &str) -> Result<i64, PortError> {
        Ok(*self.count_result.lock().unwrap())
    }
}

fn purchased_at() -> DateTime<Utc> {
    "2026-05-01T10:00:00Z".parse().unwrap()
}

#[tokio::test]
async fn register_merchant_saves_when_not_already_registered() {
    let merchants = Arc::new(StubMerchantRepository::default());
    let service = RegisterMerchantService::new(merchants.clone());
    let merchant = Merchant {
        name: "GreenGrocer".into(),
        partner: true,
    };

    service.register(merchant.clone()).await.unwrap();

    assert_eq!(*merchants.saved.lock().unwrap(), vec![merchant]);
}

#[tokio::test]
async fn register_merchant_rejects_existing_merchant() {
    let merchants = Arc::new(StubMerchantRepository {
        find_result: Mutex::new(Some(Merchant {
            name: "GreenGrocer".into(),
            partner: true,
        })),
        ..Default::default()
    });
    let service = RegisterMerchantService::new(merchants);

    let result = service
        .register(Merchant {
            name: "GreenGrocer".into(),
            partner: true,
        })
        .await;

    assert!(matches!(
        result,
        Err(ApplicationError::Domain(
            DomainError::MerchantAlreadyRegistered(_)
        ))
    ));
}

#[tokio::test]
async fn manage_categories_registers_category_and_default_rate() {
    let categories = Arc::new(StubCategoryRepository::default());
    let service = ManageProductCategoriesService::new(categories.clone());
    let category = ProductCategory {
        mcc: "5411".into(),
        name: "Groceries".into(),
        cashback_rate: dec!(0.02),
    };

    service.register(category.clone()).await.unwrap();
    service.configure_default_rate(dec!(0.005)).await.unwrap();

    assert_eq!(*categories.saved.lock().unwrap(), vec![category]);
    assert_eq!(
        *categories.configured_rates.lock().unwrap(),
        vec![dec!(0.005)]
    );
}

#[tokio::test]
async fn record_purchase_awards_cashback_at_partner_merchant() {
    let merchants = Arc::new(StubMerchantRepository {
        find_result: Mutex::new(Some(Merchant {
            name: "GreenGrocer".into(),
            partner: true,
        })),
        ..Default::default()
    });
    let categories = Arc::new(StubCategoryRepository {
        find_result: Mutex::new(Some(ProductCategory {
            mcc: "5411".into(),
            name: "Groceries".into(),
            cashback_rate: dec!(0.02),
        })),
        ..Default::default()
    });
    let cashbacks = Arc::new(StubCashbackRepository::default());

    RecordPurchaseService::new(merchants, categories, cashbacks.clone())
        .record(
            "cust-001",
            "GreenGrocer",
            "5411",
            dec!(80.00),
            purchased_at(),
        )
        .await
        .unwrap();

    let saved = cashbacks.saved.lock().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].customer_id, "cust-001");
    assert_eq!(saved[0].merchant_name, "GreenGrocer");
    assert_eq!(saved[0].product_category, "Groceries");
    assert_eq!(saved[0].cashback_amount, dec!(1.60));
}

#[tokio::test]
async fn record_purchase_ignores_non_partner_and_unregistered_merchants() {
    for merchant in [
        Merchant {
            name: "Corner Cafe".into(),
            partner: false,
        },
        Merchant {
            name: "Unknown".into(),
            partner: true,
        },
    ] {
        let merchants = Arc::new(StubMerchantRepository {
            find_result: Mutex::new((merchant.name != "Unknown").then_some(merchant)),
            ..Default::default()
        });
        let categories = Arc::new(StubCategoryRepository::default());
        let cashbacks = Arc::new(StubCashbackRepository::default());

        RecordPurchaseService::new(merchants, categories, cashbacks.clone())
            .record("cust", "shop", "5411", dec!(80.00), purchased_at())
            .await
            .unwrap();

        assert!(cashbacks.saved.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn record_purchase_below_threshold_creates_no_record() {
    let merchants = Arc::new(StubMerchantRepository::default());
    let categories = Arc::new(StubCategoryRepository::default());
    let cashbacks = Arc::new(StubCashbackRepository::default());

    RecordPurchaseService::new(merchants.clone(), categories, cashbacks.clone())
        .record("cust", "shop", "5411", dec!(0.99), purchased_at())
        .await
        .unwrap();

    assert!(cashbacks.saved.lock().unwrap().is_empty());
}

#[tokio::test]
async fn record_purchase_uses_default_rate_for_unmapped_mcc() {
    let merchants = Arc::new(StubMerchantRepository {
        find_result: Mutex::new(Some(Merchant {
            name: "Pharmacy".into(),
            partner: true,
        })),
        ..Default::default()
    });
    let categories = Arc::new(StubCategoryRepository {
        default_rate: Mutex::new(Some(dec!(0.005))),
        ..Default::default()
    });
    let cashbacks = Arc::new(StubCashbackRepository::default());

    RecordPurchaseService::new(merchants, categories, cashbacks.clone())
        .record("cust-004", "Pharmacy", "5912", dec!(100.00), purchased_at())
        .await
        .unwrap();

    let saved = cashbacks.saved.lock().unwrap();
    assert_eq!(saved[0].product_category, "Other");
    assert_eq!(saved[0].cashback_amount, dec!(0.50));
}

#[tokio::test]
async fn record_purchase_reports_missing_default_rate_as_application_error() {
    let merchants = Arc::new(StubMerchantRepository {
        find_result: Mutex::new(Some(Merchant {
            name: "Pharmacy".into(),
            partner: true,
        })),
        ..Default::default()
    });
    let categories = Arc::new(StubCategoryRepository::default());
    let cashbacks = Arc::new(StubCashbackRepository::default());

    let result = RecordPurchaseService::new(merchants, categories, cashbacks)
        .record("cust", "Pharmacy", "5912", dec!(100.00), purchased_at())
        .await;

    assert!(matches!(
        result,
        Err(ApplicationError::Domain(
            DomainError::DefaultRateNotConfigured
        ))
    ));
}

#[tokio::test]
async fn list_customer_cashback_returns_saved_records() {
    let record = CashbackRecord {
        customer_id: "cust-001".into(),
        merchant_name: "GreenGrocer".into(),
        product_category: "Groceries".into(),
        cashback_amount: dec!(2.40),
    };
    let cashbacks = Arc::new(StubCashbackRepository {
        list_result: Mutex::new(vec![record.clone()]),
        ..Default::default()
    });
    let service = ListCustomerCashbackService::new(cashbacks);

    assert_eq!(service.list_for("cust-001").await.unwrap(), vec![record]);
}

#[derive(Default)]
struct FixedTotalCashbackRepository;

#[async_trait]
impl CashbackRepository for FixedTotalCashbackRepository {
    async fn save(&self, _record: &CashbackRecord) -> Result<(), PortError> {
        Ok(())
    }

    async fn find_by_customer_id(
        &self,
        _customer_id: &str,
    ) -> Result<Vec<CashbackRecord>, PortError> {
        Ok(Vec::new())
    }

    async fn total_for_product_category(
        &self,
        product_category: &str,
    ) -> Result<Decimal, PortError> {
        assert_eq!(product_category, "Groceries");
        Ok(dec!(4.00))
    }

    async fn count_for_product_category(&self, product_category: &str) -> Result<i64, PortError> {
        assert_eq!(product_category, "Groceries");
        Ok(2)
    }
}

#[tokio::test]
async fn total_product_cashback_returns_total_and_count_with_scale_two() {
    let cashbacks = Arc::new(FixedTotalCashbackRepository);
    let service = TotalProductCashbackService::new(cashbacks);

    let total = service.total_for("Groceries").await.unwrap();

    assert_eq!(total.product, "Groceries");
    assert_eq!(total.total_cashback, dec!(4.00));
    assert_eq!(total.record_count, 2);

    let empty = ProductCashbackTotal::new("Travel".into(), dec!(0), 0);
    assert_eq!(empty.total_cashback.scale(), 2);
}

#[tokio::test]
async fn register_merchant_rejects_case_and_whitespace_variants_when_port_reports_existing() {
    for variant in [
        "greengrocer",
        "GREENGROCER",
        " GreenGrocer ",
        "  greengrocer  ",
    ] {
        let repo = Arc::new(StubMerchantRepository {
            find_result: Mutex::new(Some(Merchant {
                name: "GreenGrocer".into(),
                partner: true,
            })),
            ..Default::default()
        });
        let service = RegisterMerchantService::new(repo);
        let result = service
            .register(Merchant {
                name: variant.into(),
                partner: true,
            })
            .await;
        assert!(matches!(
            result,
            Err(ApplicationError::Domain(
                DomainError::MerchantAlreadyRegistered(_)
            ))
        ));
    }
}
