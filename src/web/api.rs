use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, Json};
use chrono::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use utoipa::{OpenApi, ToSchema};

use crate::application::{
    port::{
        ApplicationError, ListCustomerCashbackUseCase, ManageProductCategoriesUseCase,
        RecordPurchaseUseCase, RegisterMerchantUseCase, TotalProductCashbackUseCase,
    },
    ApplicationState,
};
use crate::domain::model::{Merchant, ProductCategory};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("{0}")]
    Application(#[from] ApplicationError),
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            ApiError::Validation(message) => (StatusCode::BAD_REQUEST, json!({"message": message})),
            ApiError::Application(ApplicationError::Domain(crate::domain::error::DomainError::MerchantAlreadyRegistered(_))) => (StatusCode::CONFLICT, json!({"message": self.to_string()})),
            ApiError::Application(ApplicationError::Domain(crate::domain::error::DomainError::DefaultRateNotConfigured)) => (StatusCode::UNPROCESSABLE_ENTITY, json!({"message": self.to_string()})),
            ApiError::Application(ApplicationError::Port(_)) => (StatusCode::INTERNAL_SERVER_ERROR, json!({"message": "internal server error"})),
        };
        (status, Json(body)).into_response()
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    pub customer_id: String,
    pub merchant_name: String,
    pub mcc: String,
    pub amount: String,
    pub purchased_at: String,
}
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MerchantRequest { pub name: String, pub partner: bool }
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest { pub mcc: String, pub name: String, pub cashback_rate: String }
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DefaultRateRequest { pub cashback_rate: String }

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CashbackResponse { pub merchant_name: String, pub product_category: String, pub cashback_amount: String }
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductCashbackTotalResponse { pub product: String, pub total_cashback: String, pub record_count: i64 }

pub async fn health() -> StatusCode { StatusCode::NO_CONTENT }

#[utoipa::path(
    post,
    path = "/api/merchants",
    request_body = MerchantRequest,
    responses((status = 201, description = "Merchant registered"))
)]
pub async fn register_merchant(State(state): State<ApplicationState>, Json(request): Json<MerchantRequest>) -> Result<StatusCode, ApiError> {
    let name = required_string("name", &request.name)?;
    state.register_merchant.register(Merchant { name, partner: request.partner }).await.map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    post,
    path = "/api/categories",
    request_body = CategoryRequest,
    responses((status = 201, description = "Product category registered"))
)]
pub async fn register_category(State(state): State<ApplicationState>, Json(request): Json<CategoryRequest>) -> Result<StatusCode, ApiError> {
    let mcc = required_string("mcc", &request.mcc)?;
    let name = required_string("name", &request.name)?;
    let cashback_rate = parse_decimal("cashbackRate", &request.cashback_rate)?;
    state.manage_product_categories.register(ProductCategory { mcc, name, cashback_rate }).await.map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    put,
    path = "/api/categories/default-rate",
    request_body = DefaultRateRequest,
    responses((status = 204, description = "Default rate updated"))
)]
pub async fn configure_default_rate(State(state): State<ApplicationState>, Json(request): Json<DefaultRateRequest>) -> Result<StatusCode, ApiError> {
    let rate = parse_decimal("cashbackRate", &request.cashback_rate)?;
    state.manage_product_categories.configure_default_rate(rate).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/purchases",
    request_body = PurchaseRequest,
    responses((status = 201, description = "Purchase recorded"))
)]
pub async fn record_purchase(State(state): State<ApplicationState>, Json(request): Json<PurchaseRequest>) -> Result<StatusCode, ApiError> {
    let customer_id = required_string("customerId", &request.customer_id)?;
    let merchant_name = required_string("merchantName", &request.merchant_name)?;
    let mcc = required_string("mcc", &request.mcc)?;
    let amount = parse_decimal("amount", &request.amount)?;
    DateTime::parse_from_rfc3339(&request.purchased_at).map_err(|_| ApiError::Validation("purchasedAt must be RFC-3339 date-time".into()))?;
    state.record_purchase.record(&customer_id, &merchant_name, &mcc, amount).await.map_err(ApiError::from)?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/api/customers/{customer_id}/cashback",
    params(("customer_id" = String, Path, description = "Customer identifier")),
    responses((status = 200, body = [CashbackResponse]))
)]
pub async fn list_customer_cashback(State(state): State<ApplicationState>, Path(customer_id): Path<String>) -> Result<Json<Vec<CashbackResponse>>, ApiError> {
    if customer_id.trim().is_empty() { return Err(ApiError::Validation("customerId must not be blank".into())); }
    let records = state.list_customer_cashback.list_for(&customer_id).await.map_err(ApiError::from)?;
    Ok(Json(records.into_iter().map(|r| CashbackResponse { merchant_name:r.merchant_name, product_category:r.product_category, cashback_amount: format_decimal(r.cashback_amount) }).collect()))
}

#[utoipa::path(
    get,
    path = "/api/products/{product_category}/cashback-total",
    params(("product_category" = String, Path, description = "Product category")),
    responses((status = 200, body = ProductCashbackTotalResponse))
)]
pub async fn get_product_total(State(state): State<ApplicationState>, Path(product_category): Path<String>) -> Result<Json<ProductCashbackTotalResponse>, ApiError> {
    if product_category.trim().is_empty() { return Err(ApiError::Validation("productCategory must not be blank".into())); }
    let total = state.total_product_cashback.total_for(&product_category).await.map_err(ApiError::from)?;
    Ok(Json(ProductCashbackTotalResponse { product:total.product, total_cashback:format_decimal(total.total_cashback), record_count:total.record_count }))
}

fn required_string(field: &str, value: &str) -> Result<String, ApiError> {
    if value.trim().is_empty() { return Err(ApiError::Validation(format!("{field} must not be blank"))); }
    Ok(value.to_string())
}
fn parse_decimal(field: &str, value: &str) -> Result<Decimal, ApiError> {
    value.parse::<Decimal>().map_err(|_| ApiError::Validation(format!("{field} must be a decimal")))
}
fn format_decimal(value: Decimal) -> String { value.round_dp(2).to_string() }

#[derive(OpenApi)]
#[openapi(
    info(title = "Cashback Rewards API", version = "0.1.0"),
    paths(register_merchant, register_category, configure_default_rate, record_purchase, list_customer_cashback, get_product_total),
    components(schemas(PurchaseRequest, MerchantRequest, CategoryRequest, DefaultRateRequest, CashbackResponse, ProductCashbackTotalResponse))
)]
pub struct ApiDoc;
