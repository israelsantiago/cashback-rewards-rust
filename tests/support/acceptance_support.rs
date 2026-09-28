use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use cashback_rewards_rust::{
    adapter::{
        r#in::web::{self, WebState},
        out::persistence::{PgCashbackRepository, PgCategoryRepository, PgMerchantRepository},
    },
    application::{
        port::out::{CashbackRepository, CategoryRepository, MerchantRepository},
        service::{
            ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
            RegisterMerchantService, TotalProductCashbackService,
        },
    },
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
pub fn build_app(pool: sqlx::PgPool) -> axum::Router {
    let merchants: Arc<dyn MerchantRepository> = Arc::new(PgMerchantRepository::new(pool.clone()));
    let categories: Arc<dyn CategoryRepository> = Arc::new(PgCategoryRepository::new(pool.clone()));
    let cashbacks: Arc<dyn CashbackRepository> = Arc::new(PgCashbackRepository::new(pool));
    web::router(WebState::new(
        Arc::new(ListCustomerCashbackService::new(cashbacks.clone())),
        Arc::new(ManageProductCategoriesService::new(categories.clone())),
        Arc::new(RecordPurchaseService::new(
            merchants.clone(),
            categories,
            cashbacks.clone(),
        )),
        Arc::new(RegisterMerchantService::new(merchants)),
        Arc::new(TotalProductCashbackService::new(cashbacks)),
    ))
}

pub async fn request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
) -> Result<axum::response::Response, Box<dyn std::error::Error>> {
    Ok(app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))?,
        )
        .await?)
}

pub async fn cashback_for(
    app: axum::Router,
    customer: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let r = request(
        app,
        "GET",
        &format!("/api/customers/{customer}/cashback"),
        json!({}),
    )
    .await?;
    assert_eq!(r.status(), StatusCode::OK);
    Ok(serde_json::from_slice(
        &to_bytes(r.into_body(), usize::MAX).await?,
    )?)
}
