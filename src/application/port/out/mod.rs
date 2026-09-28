use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::model::{CashbackRecord, Merchant, ProductCategory};

#[derive(Debug, Error)]
pub enum PortError {
    #[error("persistence error: {0}")]
    Database(String),
}

#[async_trait]
pub trait CashbackRepository: Send + Sync {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError>;
    async fn find_by_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CashbackRecord>, PortError>;
    async fn total_for_product_category(
        &self,
        product_category: &str,
    ) -> Result<Decimal, PortError>;
    async fn count_for_product_category(&self, product_category: &str) -> Result<i64, PortError>;
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn save(&self, category: &ProductCategory) -> Result<(), PortError>;
    async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError>;
    async fn default_rate(&self) -> Result<Option<Decimal>, PortError>;
    async fn find_by_mcc(&self, mcc: &str) -> Result<Option<ProductCategory>, PortError>;
}

#[async_trait]
pub trait MerchantRepository: Send + Sync {
    async fn save(&self, merchant: &Merchant) -> Result<(), PortError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Merchant>, PortError>;
}
