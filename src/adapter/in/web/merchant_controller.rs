use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::adapter::r#in::web::{WebState, error::ApiError};
use crate::domain::model::Merchant;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MerchantRequest {
    pub name: String,
    pub partner: bool,
}

#[utoipa::path(
    post,
    path = "/api/merchants",
    request_body = MerchantRequest,
    responses((status = 201, description = "Merchant registered"))
)]
pub async fn register_merchant(
    State(state): State<WebState>,
    Json(request): Json<MerchantRequest>,
) -> Result<StatusCode, ApiError> {
    let name = required_string("name", &request.name)?;
    state
        .register_merchant
        .register(Merchant {
            name,
            partner: request.partner,
        })
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
