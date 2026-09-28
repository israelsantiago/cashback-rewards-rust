use axum::{
    Json,
    extract::{Path, State},
};
use rust_decimal::Decimal;
use serde::Serialize;
use utoipa::ToSchema;

use crate::adapter::r#in::web::{WebState, error::ApiError};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CashbackResponse {
    pub merchant_name: String,
    pub product_category: String,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub cashback_amount: Decimal,
}

#[utoipa::path(
    get,
    path = "/api/customers/{customer_id}/cashback",
    params(("customer_id" = String, Path, description = "Customer identifier")),
    responses((status = 200, body = [CashbackResponse]))
)]
pub async fn list_customer_cashback(
    State(state): State<WebState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<CashbackResponse>>, ApiError> {
    let records = state
        .list_cashback
        .list_for(&customer_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(
        records
            .into_iter()
            .map(|record| CashbackResponse {
                merchant_name: record.merchant_name,
                product_category: record.product_category,
                cashback_amount: record.cashback_amount,
            })
            .collect(),
    ))
}
