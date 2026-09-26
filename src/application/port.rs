use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::domain::{
    error::DomainError,
    model::{CashbackRecord, Merchant, ProductCategory, ProductCashbackTotal},
};

#[derive(Debug, Error)]
pub enum PortError {
    #[error("persistence error: {0}")]
    Database(String),
}

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Port(#[from] PortError),
}

#[async_trait]
pub trait CashbackRepository: Send + Sync {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError>;
    async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, PortError>;
    async fn total_for_product_category(&self, product_category: &str) -> Result<Decimal, PortError>;
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

pub type CashbackRepo = Arc<dyn CashbackRepository>;
pub type CategoryRepo = Arc<dyn CategoryRepository>;
pub type MerchantRepo = Arc<dyn MerchantRepository>;

#[async_trait]
pub trait ListCustomerCashbackUseCase: Send + Sync {
    async fn list_for(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, ApplicationError>;
}

#[async_trait]
pub trait ManageProductCategoriesUseCase: Send + Sync {
    async fn register(&self, category: ProductCategory) -> Result<(), ApplicationError>;
    async fn configure_default_rate(&self, rate: Decimal) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait RecordPurchaseUseCase: Send + Sync {
    async fn record(&self, customer_id: &str, merchant_name: &str, mcc: &str, amount: Decimal)
        -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait RegisterMerchantUseCase: Send + Sync {
    async fn register(&self, merchant: Merchant) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait TotalProductCashbackUseCase: Send + Sync {
    async fn total_for(&self, product_category: &str) -> Result<ProductCashbackTotal, ApplicationError>;
}
