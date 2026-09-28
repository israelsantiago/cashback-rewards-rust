use std::sync::Arc;

use rust_decimal::Decimal;

use crate::application::ApplicationError;
use crate::application::port::r#in::ManageProductCategoriesUseCase;
use crate::application::port::out::CategoryRepository;
use crate::domain::model::ProductCategory;

pub struct ManageProductCategoriesService {
    categories: Arc<dyn CategoryRepository>,
}

impl ManageProductCategoriesService {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
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
