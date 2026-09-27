pub mod api;

use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::ApplicationState;
use api::{
    ApiDoc, configure_default_rate, get_product_total, health, list_customer_cashback,
    record_purchase, register_category, register_merchant,
};

pub fn router(state: ApplicationState) -> Router {
    Router::new()
        .route("/health", get(health))
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
