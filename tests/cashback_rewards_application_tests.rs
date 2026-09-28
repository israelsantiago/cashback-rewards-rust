use async_trait::async_trait;
use cashback_rewards_rust::{
    adapter::r#in::web::{self, WebState},
    application::{
        ApplicationError,
        port::r#in::{
            ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
            RegisterMerchantUseCase, TotalProductCashbackUseCase,
        },
    },
    domain::model::{CashbackRecord, Merchant, ProductCashbackTotal, ProductCategory},
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

struct Noop;
#[async_trait]
impl ListCustomerCashbackUseCase for Noop {
    async fn list_for(&self, _: &str) -> Result<Vec<CashbackRecord>, ApplicationError> {
        Ok(vec![])
    }
}
#[async_trait]
impl ManageProductCategoriesUseCase for Noop {
    async fn register(&self, _: ProductCategory) -> Result<(), ApplicationError> {
        Ok(())
    }
    async fn configure_default_rate(&self, _: Decimal) -> Result<(), ApplicationError> {
        Ok(())
    }
}
#[async_trait]
impl RecordPurchaseUseCase for Noop {
    async fn record(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: Decimal,
        _: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}
#[async_trait]
impl RegisterMerchantUseCase for Noop {
    async fn register(&self, _: Merchant) -> Result<(), ApplicationError> {
        Ok(())
    }
}
#[async_trait]
impl TotalProductCashbackUseCase for Noop {
    async fn total_for(&self, p: &str) -> Result<ProductCashbackTotal, ApplicationError> {
        Ok(ProductCashbackTotal::new(p.into(), Decimal::ZERO, 0))
    }
}

#[test]
fn context_loads_as_a_composable_hexagonal_graph() {
    let n = Arc::new(Noop);
    let _router = web::router(WebState::new(n.clone(), n.clone(), n.clone(), n.clone(), n));
}
