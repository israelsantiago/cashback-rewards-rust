use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::port::r#in::TotalProductCashbackUseCase;
use crate::application::port::out::CashbackRepository;
use crate::domain::model::ProductCashbackTotal;

pub struct TotalProductCashbackService {
    cashbacks: Arc<dyn CashbackRepository>,
}

impl TotalProductCashbackService {
    pub fn new(cashbacks: Arc<dyn CashbackRepository>) -> Self {
        Self { cashbacks }
    }
}

#[async_trait::async_trait]
impl TotalProductCashbackUseCase for TotalProductCashbackService {
    async fn total_for(
        &self,
        product_category: &str,
    ) -> Result<ProductCashbackTotal, ApplicationError> {
        let total = self
            .cashbacks
            .total_for_product_category(product_category)
            .await?;
        let count = self
            .cashbacks
            .count_for_product_category(product_category)
            .await?;

        Ok(ProductCashbackTotal::new(
            product_category.to_owned(),
            total,
            count,
        ))
    }
}
