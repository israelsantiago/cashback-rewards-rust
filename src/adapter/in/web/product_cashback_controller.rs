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
pub struct ProductCashbackTotalResponse {
    pub product: String,
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    pub total_cashback: Decimal,
    pub record_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/products/{product_category}/cashback-total",
    params(("product_category" = String, Path, description = "Product category")),
    responses((status = 200, body = ProductCashbackTotalResponse))
)]
pub async fn get_product_total(
    State(state): State<WebState>,
    Path(product_category): Path<String>,
) -> Result<Json<ProductCashbackTotalResponse>, ApiError> {
    let total = state
        .total_product_cashback
        .total_for(&product_category)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ProductCashbackTotalResponse {
        product: total.product,
        total_cashback: total.total_cashback,
        record_count: total.record_count,
    }))
}
