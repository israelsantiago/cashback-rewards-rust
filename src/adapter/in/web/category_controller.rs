use axum::{Json, extract::State, http::StatusCode};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adapter::r#in::web::{WebState, error::ApiError};
use crate::domain::model::ProductCategory;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    pub mcc: String,
    pub name: String,
    pub cashback_rate: Decimal,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DefaultRateRequest {
    pub cashback_rate: Decimal,
}

#[utoipa::path(
    post,
    path = "/api/categories",
    request_body = CategoryRequest,
    responses((status = 201, description = "Product category registered"))
)]
pub async fn register_category(
    State(state): State<WebState>,
    Json(request): Json<CategoryRequest>,
) -> Result<StatusCode, ApiError> {
    let mcc = required_string("mcc", &request.mcc)?;
    let name = required_string("name", &request.name)?;

    state
        .manage_categories
        .register(ProductCategory {
            mcc,
            name,
            cashback_rate: request.cashback_rate,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    put,
    path = "/api/categories/default-rate",
    request_body = DefaultRateRequest,
    responses((status = 204, description = "Default rate updated"))
)]
pub async fn configure_default_rate(
    State(state): State<WebState>,
    Json(request): Json<DefaultRateRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .manage_categories
        .configure_default_rate(request.cashback_rate)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

fn required_string(field: &str, value: &str) -> Result<String, ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::Validation(format!("{field} must not be blank")));
    }
    Ok(value.to_string())
}
