use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use cashback_rewards_rust::{
    bootstrap::build_app,
    domain::model::{Merchant, ProductCategory},
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::support::fixtures::{FixtureIdentity, TestTimer, parse_purchase_timestamp};

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

pub struct AcceptanceFixture {
    identity: FixtureIdentity,
    app: axum::Router,
    default_rate_guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

pub struct CategoryFixture {
    pub category: ProductCategory,
}

pub struct MerchantFixture {
    pub merchant: Merchant,
}

impl AcceptanceFixture {
    pub fn new(pool: PgPool, scenario: &str) -> Self {
        let identity = FixtureIdentity::new(scenario);
        Self {
            app: build_app(pool),
            identity,
            default_rate_guard: None,
        }
    }

    pub fn id(&self) -> String {
        self.identity.id()
    }

    pub fn timer(&self, test: &str) -> TestTimer {
        TestTimer::new("acceptance", test, self.id())
    }

    pub fn identity(&self) -> &FixtureIdentity {
        &self.identity
    }

    pub async fn register_category(
        &self,
        base: &str,
        cashback_rate: Decimal,
    ) -> Result<CategoryFixture, Box<dyn std::error::Error>> {
        let category = self.identity.category(base, cashback_rate);
        let response = request(
            self.app.clone(),
            "POST",
            "/api/categories",
            json!({
                "mcc": &category.mcc,
                "name": &category.name,
                "cashbackRate": category.cashback_rate,
            }),
        )
        .await?;
        assert_eq!(response.status(), StatusCode::CREATED);
        Ok(CategoryFixture { category })
    }

    pub async fn configure_default_rate(
        &mut self,
        cashback_rate: Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.default_rate_guard.is_none() {
            self.default_rate_guard = Some(crate::support::postgres::default_rate_guard().await);
        }
        let response = request(
            self.app.clone(),
            "PUT",
            "/api/categories/default-rate",
            json!({ "cashbackRate": cashback_rate }),
        )
        .await?;
        assert!(response.status().is_success());
        Ok(())
    }

    pub async fn register_merchant(
        &self,
        base: &str,
        partner: bool,
    ) -> Result<MerchantFixture, Box<dyn std::error::Error>> {
        let merchant = self.identity.merchant(base, partner);
        let response = request(
            self.app.clone(),
            "POST",
            "/api/merchants",
            json!({ "name": &merchant.name, "partner": merchant.partner }),
        )
        .await?;
        assert_eq!(response.status(), StatusCode::CREATED);
        Ok(MerchantFixture { merchant })
    }

    pub async fn record_purchase(
        &self,
        customer_base: &str,
        merchant: &MerchantFixture,
        category: &CategoryFixture,
        amount: Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = request(
            self.app.clone(),
            "POST",
            "/api/purchases",
            json!({
                "customerId": self.identity.customer_id(customer_base),
                "merchantName": &merchant.merchant.name,
                "mcc": &category.category.mcc,
                "amount": amount,
                "purchasedAt": parse_purchase_timestamp(),
            }),
        )
        .await?;
        assert!(response.status().is_success());
        Ok(())
    }

    pub async fn record_purchase_with_mcc(
        &self,
        customer_base: &str,
        merchant: &MerchantFixture,
        mcc: &str,
        amount: Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response = request(
            self.app.clone(),
            "POST",
            "/api/purchases",
            json!({
                "customerId": self.identity.customer_id(customer_base),
                "merchantName": &merchant.merchant.name,
                "mcc": mcc,
                "amount": amount,
                "purchasedAt": parse_purchase_timestamp(),
            }),
        )
        .await?;
        assert!(response.status().is_success());
        Ok(())
    }

    pub async fn cashback_for(
        &self,
        customer_base: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let customer_id = self.identity.customer_id(customer_base);
        let response = request(
            self.app.clone(),
            "GET",
            &format!("/api/customers/{customer_id}/cashback"),
            json!({}),
        )
        .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX).await?,
        )?)
    }

    pub fn category_without_registration(&self, base: &str) -> CategoryFixture {
        CategoryFixture {
            category: self.identity.category(base, Decimal::ZERO),
        }
    }

    pub async fn product_total(
        &self,
        category: &CategoryFixture,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let response = request(
            self.app.clone(),
            "GET",
            &format!("/api/products/{}/cashback-total", category.category.name),
            json!({}),
        )
        .await?;
        assert_eq!(response.status(), StatusCode::OK);
        Ok(serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX).await?,
        )?)
    }
}
