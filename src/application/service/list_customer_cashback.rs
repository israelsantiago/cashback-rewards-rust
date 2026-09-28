use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::port::r#in::ListCustomerCashbackUseCase;
use crate::application::port::out::CashbackRepository;
use crate::domain::model::CashbackRecord;

pub struct ListCustomerCashbackService {
    cashbacks: Arc<dyn CashbackRepository>,
}

impl ListCustomerCashbackService {
    pub fn new(cashbacks: Arc<dyn CashbackRepository>) -> Self {
        Self { cashbacks }
    }
}

#[async_trait::async_trait]
impl ListCustomerCashbackUseCase for ListCustomerCashbackService {
    async fn list_for(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, ApplicationError> {
        Ok(self.cashbacks.find_by_customer_id(customer_id).await?)
    }
}
