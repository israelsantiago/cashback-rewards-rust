use rust_decimal::Decimal;

use crate::application::port::{
    ApplicationError, CashbackRepo, CategoryRepo, ListCustomerCashbackUseCase,
    ManageProductCategoriesUseCase, MerchantRepo, RecordPurchaseUseCase, RegisterMerchantUseCase,
    TotalProductCashbackUseCase,
};

use crate::domain::{
    error::DomainError,
    model::{
        CashbackRecord, Merchant, MinimumPurchaseThreshold, ProductCashbackTotal, ProductCategory,
    },
    service::calculate_cashback,
};

#[derive(Clone)]
pub struct ListCustomerCashbackService {
    cashbacks: CashbackRepo,
}

impl ListCustomerCashbackService {
    pub fn new(cashbacks: CashbackRepo) -> Self {
        Self { cashbacks }
    }
}

#[async_trait::async_trait]
impl ListCustomerCashbackUseCase for ListCustomerCashbackService {
    async fn list_for(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, ApplicationError> {
        Ok(self.cashbacks.find_by_customer_id(customer_id).await?)
    }
}

#[derive(Clone)]
pub struct ManageProductCategoriesService {
    categories: CategoryRepo,
}

impl ManageProductCategoriesService {
    pub fn new(categories: CategoryRepo) -> Self {
        Self { categories }
    }
}

#[async_trait::async_trait]
impl ManageProductCategoriesUseCase for ManageProductCategoriesService {
    async fn register(&self, category: ProductCategory) -> Result<(), ApplicationError> {
        Ok(self.categories.save(&category).await?)
    }

    async fn configure_default_rate(&self, rate: Decimal) -> Result<(), ApplicationError> {
        Ok(self.categories.save_default_rate(rate).await?)
    }
}

#[derive(Clone)]
pub struct RegisterMerchantService {
    merchants: MerchantRepo,
}

impl RegisterMerchantService {
    pub fn new(merchants: MerchantRepo) -> Self {
        Self { merchants }
    }
}

#[async_trait::async_trait]
impl RegisterMerchantUseCase for RegisterMerchantService {
    async fn register(&self, merchant: Merchant) -> Result<(), ApplicationError> {
        if self.merchants.find_by_name(&merchant.name).await?.is_some() {
            return Err(DomainError::MerchantAlreadyRegistered(merchant.name).into());
        }

        Ok(self.merchants.save(&merchant).await?)
    }
}

#[derive(Clone)]
pub struct TotalProductCashbackService {
    cashbacks: CashbackRepo,
}

impl TotalProductCashbackService {
    pub fn new(cashbacks: CashbackRepo) -> Self {
        Self { cashbacks }
    }
}

#[async_trait::async_trait]
impl TotalProductCashbackUseCase for TotalProductCashbackService {
    async fn total_for(
        &self,
        product_category: &str,
    ) -> Result<ProductCashbackTotal, ApplicationError> {
        let total = self.cashbacks.total_for_product_category(product_category);
        let count = self.cashbacks.count_for_product_category(product_category);
        let (total, count) = tokio::join!(total, count);

        Ok(ProductCashbackTotal::new(
            product_category.to_owned(),
            total?,
            count?,
        ))
    }
}

#[derive(Clone)]
pub struct RecordPurchaseService {
    merchants: MerchantRepo,
    categories: CategoryRepo,
    cashbacks: CashbackRepo,
}

impl RecordPurchaseService {
    pub fn new(merchants: MerchantRepo, categories: CategoryRepo, cashbacks: CashbackRepo) -> Self {
        Self {
            merchants,
            categories,
            cashbacks,
        }
    }
}

#[async_trait::async_trait]
impl RecordPurchaseUseCase for RecordPurchaseService {
    async fn record(
        &self,
        customer_id: &str,
        merchant_name: &str,
        mcc: &str,
        amount: Decimal,
    ) -> Result<(), ApplicationError> {
        if !MinimumPurchaseThreshold::default().is_met_by(amount) {
            return Ok(());
        }

        let Some(merchant) = self.merchants.find_by_name(merchant_name).await? else {
            return Ok(());
        };

        if !merchant.partner {
            return Ok(());
        }

        let category = match self.categories.find_by_mcc(mcc).await? {
            Some(category) => category,
            None => {
                let rate = self
                    .categories
                    .default_rate()
                    .await?
                    .ok_or(DomainError::DefaultRateNotConfigured)?;
                ProductCategory::unmapped(mcc, rate)
            }
        };

        let cashback_amount = calculate_cashback(amount, category.cashback_rate);
        let record = CashbackRecord {
            customer_id: customer_id.to_owned(),
            merchant_name: merchant_name.to_owned(),
            product_category: category.name,
            cashback_amount,
        };

        Ok(self.cashbacks.save(&record).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::port::{
        CashbackRepository, CategoryRepository, MerchantRepository, PortError,
    };
    use async_trait::async_trait;
    use rust_decimal::dec;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    struct Merchants(Mutex<HashMap<String, Merchant>>);
    struct Categories {
        by_mcc: Mutex<HashMap<String, ProductCategory>>,
        default: Mutex<Option<Decimal>>,
    }
    struct Cashbacks(Mutex<Vec<CashbackRecord>>);

    #[async_trait]
    impl MerchantRepository for Merchants {
        async fn save(&self, merchant: &Merchant) -> Result<(), PortError> {
            self.0
                .lock()
                .unwrap()
                .insert(merchant.name.trim().to_lowercase(), merchant.clone());
            Ok(())
        }

        async fn find_by_name(&self, name: &str) -> Result<Option<Merchant>, PortError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .get(&name.trim().to_lowercase())
                .cloned())
        }
    }

    #[async_trait]
    impl CategoryRepository for Categories {
        async fn save(&self, category: &ProductCategory) -> Result<(), PortError> {
            self.by_mcc
                .lock()
                .unwrap()
                .insert(category.mcc.clone(), category.clone());
            Ok(())
        }

        async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError> {
            *self.default.lock().unwrap() = Some(rate);
            Ok(())
        }

        async fn default_rate(&self) -> Result<Option<Decimal>, PortError> {
            Ok(*self.default.lock().unwrap())
        }

        async fn find_by_mcc(&self, mcc: &str) -> Result<Option<ProductCategory>, PortError> {
            Ok(self.by_mcc.lock().unwrap().get(mcc).cloned())
        }
    }

    #[async_trait]
    impl CashbackRepository for Cashbacks {
        async fn save(&self, record: &CashbackRecord) -> Result<(), PortError> {
            self.0.lock().unwrap().push(record.clone());
            Ok(())
        }

        async fn find_by_customer_id(
            &self,
            customer_id: &str,
        ) -> Result<Vec<CashbackRecord>, PortError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|record| record.customer_id == customer_id)
                .cloned()
                .collect())
        }

        async fn total_for_product_category(
            &self,
            product_category: &str,
        ) -> Result<Decimal, PortError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|record| record.product_category == product_category)
                .map(|record| record.cashback_amount)
                .sum())
        }

        async fn count_for_product_category(
            &self,
            product_category: &str,
        ) -> Result<i64, PortError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|record| record.product_category == product_category)
                .count() as i64)
        }
    }

    #[tokio::test]
    async fn partner_purchase_records_cashback() {
        let merchants = Arc::new(Merchants(Mutex::new(HashMap::new())));
        let categories = Arc::new(Categories {
            by_mcc: Mutex::new(HashMap::new()),
            default: Mutex::new(None),
        });
        let cashbacks = Arc::new(Cashbacks(Mutex::new(Vec::new())));

        merchants
            .save(&Merchant {
                name: "GreenGrocer".into(),
                partner: true,
            })
            .await
            .unwrap();
        categories
            .save(&ProductCategory {
                mcc: "5411".into(),
                name: "Groceries".into(),
                cashback_rate: dec!(0.02),
            })
            .await
            .unwrap();

        let service = RecordPurchaseService::new(merchants, categories, cashbacks.clone());
        service
            .record("cust-1", "GreenGrocer", "5411", dec!(80.00))
            .await
            .unwrap();

        assert_eq!(cashbacks.0.lock().unwrap()[0].cashback_amount, dec!(1.60));
    }

    #[tokio::test]
    async fn below_threshold_creates_no_record() {
        let merchants = Arc::new(Merchants(Mutex::new(HashMap::new())));
        let categories = Arc::new(Categories {
            by_mcc: Mutex::new(HashMap::new()),
            default: Mutex::new(None),
        });
        let cashbacks = Arc::new(Cashbacks(Mutex::new(Vec::new())));

        merchants
            .save(&Merchant {
                name: "Tiny".into(),
                partner: true,
            })
            .await
            .unwrap();
        categories
            .save(&ProductCategory {
                mcc: "5411".into(),
                name: "Groceries".into(),
                cashback_rate: dec!(0.02),
            })
            .await
            .unwrap();

        let service = RecordPurchaseService::new(merchants, categories, cashbacks.clone());
        service
            .record("cust-1", "Tiny", "5411", dec!(0.99))
            .await
            .unwrap();

        assert!(cashbacks.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unmapped_mcc_uses_default_rate() {
        let merchants = Arc::new(Merchants(Mutex::new(HashMap::new())));
        let categories = Arc::new(Categories {
            by_mcc: Mutex::new(HashMap::new()),
            default: Mutex::new(Some(dec!(0.005))),
        });
        let cashbacks = Arc::new(Cashbacks(Mutex::new(Vec::new())));

        merchants
            .save(&Merchant {
                name: "Pharmacy".into(),
                partner: true,
            })
            .await
            .unwrap();

        let service = RecordPurchaseService::new(merchants, categories, cashbacks.clone());
        service
            .record("cust-1", "Pharmacy", "5912", dec!(100))
            .await
            .unwrap();

        let records = cashbacks.0.lock().unwrap();
        assert_eq!(records[0].product_category, "Other");
        assert_eq!(records[0].cashback_amount, dec!(0.50));
    }
}
