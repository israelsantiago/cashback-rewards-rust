pub mod port;
pub mod service;

use service::{
    ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
    RegisterMerchantService, TotalProductCashbackService,
};

#[derive(Clone)]
pub struct ApplicationState {
    pub list_cashback: ListCustomerCashbackService,
    pub manage_categories: ManageProductCategoriesService,
    pub record_purchase: RecordPurchaseService,
    pub register_merchant: RegisterMerchantService,
    pub total_product_cashback: TotalProductCashbackService,
}

impl ApplicationState {
    pub fn new(
        list_cashback: ListCustomerCashbackService,
        manage_categories: ManageProductCategoriesService,
        record_purchase: RecordPurchaseService,
        register_merchant: RegisterMerchantService,
        total_product_cashback: TotalProductCashbackService,
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
