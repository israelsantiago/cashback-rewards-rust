use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    adapter::{
        r#in::web::{self, WebState},
        out::persistence::{PgCashbackRepository, PgCategoryRepository, PgMerchantRepository},
    },
    application::{
        port::r#in::{
            ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
            RegisterMerchantUseCase, TotalProductCashbackUseCase,
        },
        service::{
            ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
            RegisterMerchantService, TotalProductCashbackService,
        },
    },
};

/// Builds the application using the same adapter wiring used in production.
///
/// Only the infrastructure resource (`PgPool`) is supplied by the caller,
/// allowing tests to provide a real PostgreSQL Testcontainer while keeping
/// the application composition identical to production.
pub fn build_app(pool: PgPool) -> Router {
    let merchants: Arc<dyn crate::application::port::out::MerchantRepository> =
        Arc::new(PgMerchantRepository::new(pool.clone()));
    let categories: Arc<dyn crate::application::port::out::CategoryRepository> =
        Arc::new(PgCategoryRepository::new(pool.clone()));
    let cashbacks: Arc<dyn crate::application::port::out::CashbackRepository> =
        Arc::new(PgCashbackRepository::new(pool));

    let list_cashback: Arc<dyn ListCustomerCashbackUseCase> =
        Arc::new(ListCustomerCashbackService::new(cashbacks.clone()));
    let manage_categories: Arc<dyn ManageProductCategoriesUseCase> =
        Arc::new(ManageProductCategoriesService::new(categories.clone()));
    let record_purchase: Arc<dyn RecordPurchaseUseCase> = Arc::new(RecordPurchaseService::new(
        merchants.clone(),
        categories,
        cashbacks.clone(),
    ));
    let register_merchant: Arc<dyn RegisterMerchantUseCase> =
        Arc::new(RegisterMerchantService::new(merchants));
    let total_product_cashback: Arc<dyn TotalProductCashbackUseCase> =
        Arc::new(TotalProductCashbackService::new(cashbacks));

    web::router(WebState::new(
        list_cashback,
        manage_categories,
        record_purchase,
        register_merchant,
        total_product_cashback,
    ))
}
