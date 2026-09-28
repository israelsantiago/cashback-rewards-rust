use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adapter::r#in::web::{WebState, error::ApiError};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    pub customer_id: String,
    pub merchant_name: String,
    pub mcc: String,
    pub amount: Decimal,
    pub purchased_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/api/purchases",
    request_body = PurchaseRequest,
    responses((status = 201, description = "Purchase recorded"))
)]
pub async fn record_purchase(
    State(state): State<WebState>,
    Json(request): Json<PurchaseRequest>,
) -> Result<StatusCode, ApiError> {
    let customer_id = required_string("customerId", &request.customer_id)?;
    let merchant_name = required_string("merchantName", &request.merchant_name)?;
    let mcc = required_string("mcc", &request.mcc)?;

    state
        .record_purchase
        .record(
            &customer_id,
            &merchant_name,
            &mcc,
            request.amount,
            request.purchased_at,
        )
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::CREATED)
}

fn required_string(field: &str, value: &str) -> Result<String, ApiError> {
    if value.trim().is_empty() {
        return Err(ApiError::Validation(format!("{field} must not be blank")));
    }
    Ok(value.to_string())
}
