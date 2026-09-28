use std::sync::Arc;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::application::ApplicationError;
use crate::application::port::r#in::RecordPurchaseUseCase;
use crate::application::port::out::{CashbackRepository, CategoryRepository, MerchantRepository};
use crate::domain::exception::DomainError;
use crate::domain::model::{CashbackRecord, MinimumPurchaseThreshold, ProductCategory};
use crate::domain::service::CashbackCalculator;

pub struct RecordPurchaseService {
    merchants: Arc<dyn MerchantRepository>,
    categories: Arc<dyn CategoryRepository>,
    cashbacks: Arc<dyn CashbackRepository>,
}

impl RecordPurchaseService {
    pub fn new(
        merchants: Arc<dyn MerchantRepository>,
        categories: Arc<dyn CategoryRepository>,
        cashbacks: Arc<dyn CashbackRepository>,
    ) -> Self {
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
        purchased_at: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let _ = purchased_at; // Java receives Instant but intentionally does not persist it.

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

        let cashback_amount = CashbackCalculator::calculate(amount, category.cashback_rate);
        let record = CashbackRecord {
            customer_id: customer_id.to_owned(),
            merchant_name: merchant_name.to_owned(),
            product_category: category.name,
            cashback_amount,
        };

        Ok(self.cashbacks.save(&record).await?)
    }
}
