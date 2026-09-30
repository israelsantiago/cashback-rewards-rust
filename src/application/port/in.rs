use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::application::ApplicationError;
use crate::domain::model::{CashbackRecord, Merchant, ProductCashbackTotal, ProductCategory};

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
    async fn record(
        &self,
        customer_id: &str,
        merchant_name: &str,
        mcc: &str,
        amount: Decimal,
        purchased_at: DateTime<Utc>,
    ) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait RegisterMerchantUseCase: Send + Sync {
    async fn register(&self, merchant: Merchant) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait TotalProductCashbackUseCase: Send + Sync {
    async fn total_for(
        &self,
        product_category: &str,
    ) -> Result<ProductCashbackTotal, ApplicationError>;
}
