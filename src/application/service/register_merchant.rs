use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::port::r#in::RegisterMerchantUseCase;
use crate::application::port::out::MerchantRepository;
use crate::domain::exception::DomainError;
use crate::domain::model::Merchant;

pub struct RegisterMerchantService {
    merchants: Arc<dyn MerchantRepository>,
}

impl RegisterMerchantService {
    pub fn new(merchants: Arc<dyn MerchantRepository>) -> Self {
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
