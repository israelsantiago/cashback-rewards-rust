use std::sync::Arc;

use crate::application::port::r#in::{
    ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
    RegisterMerchantUseCase, TotalProductCashbackUseCase,
};

#[derive(Clone)]
pub struct WebState {
    pub list_cashback: Arc<dyn ListCustomerCashbackUseCase>,
    pub manage_categories: Arc<dyn ManageProductCategoriesUseCase>,
    pub record_purchase: Arc<dyn RecordPurchaseUseCase>,
    pub register_merchant: Arc<dyn RegisterMerchantUseCase>,
    pub total_product_cashback: Arc<dyn TotalProductCashbackUseCase>,
}

impl WebState {
    pub fn new(
        list_cashback: Arc<dyn ListCustomerCashbackUseCase>,
        manage_categories: Arc<dyn ManageProductCategoriesUseCase>,
        record_purchase: Arc<dyn RecordPurchaseUseCase>,
        register_merchant: Arc<dyn RegisterMerchantUseCase>,
        total_product_cashback: Arc<dyn TotalProductCashbackUseCase>,
    ) -> Self {
        Self {
            list_cashback,
            manage_categories,
            record_purchase,
            register_merchant,
            total_product_cashback,
        }
    }
}
