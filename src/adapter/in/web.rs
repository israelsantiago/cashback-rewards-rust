pub mod cashback_controller;
pub mod category_controller;
mod error;
pub mod merchant_controller;
pub mod product_cashback_controller;
pub mod purchase_controller;
mod state;

use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use cashback_controller::{CashbackResponse, list_customer_cashback};
use category_controller::{
    CategoryRequest, DefaultRateRequest, configure_default_rate, register_category,
};
use merchant_controller::{MerchantRequest, register_merchant};
use product_cashback_controller::{ProductCashbackTotalResponse, get_product_total};
use purchase_controller::{PurchaseRequest, record_purchase};
pub use state::WebState;

pub fn router(state: WebState) -> Router {
    Router::new()
        .route(
            "/api/customers/{customer_id}/cashback",
            get(list_customer_cashback),
        )
        .route(
            "/api/products/{product_category}/cashback-total",
            get(get_product_total),
        )
        .route("/api/purchases", post(record_purchase))
        .route("/api/merchants", post(register_merchant))
        .route("/api/categories", post(register_category))
        .route("/api/categories/default-rate", put(configure_default_rate))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Cashback Rewards API", version = "0.1.0"),
    paths(
        merchant_controller::register_merchant,
        category_controller::register_category,
        category_controller::configure_default_rate,
        purchase_controller::record_purchase,
        cashback_controller::list_customer_cashback,
        product_cashback_controller::get_product_total
    ),
    components(schemas(
        PurchaseRequest,
        MerchantRequest,
        CategoryRequest,
        DefaultRateRequest,
        CashbackResponse,
        ProductCashbackTotalResponse
    ))
)]
pub struct ApiDoc;
