use std::time::Instant;

use cashback_rewards_rust::domain::model::{CashbackRecord, Merchant, ProductCategory};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FixtureIdentity {
    scenario: String,
    uuid: Uuid,
    token: String,
}

impl FixtureIdentity {
    pub fn new(scenario: impl Into<String>) -> Self {
        let scenario = scenario.into();
        let uuid = Uuid::new_v4();
        let token = uuid.simple().to_string()[..10].to_string();
        let identity = Self {
            scenario,
            uuid,
            token,
        };
        info!(
            target: "cashback_rewards_rust::test_timing",
            event = "fixture.created",
            fixture = %identity.id(),
            scenario = %identity.scenario,
        );
        identity
    }

    pub fn id(&self) -> String {
        format!("{}-{}", self.scenario, self.token)
    }

    pub fn merchant_name(&self, base: &str) -> String {
        format!("{base}-{}", self.token)
    }

    pub fn customer_id(&self, base: &str) -> String {
        format!("{base}-{}", self.token)
    }

    pub fn category_name(&self, base: &str) -> String {
        format!("{base}-{}", self.token)
    }

    /// Produces an 8-digit numeric MCC-shaped identifier unique to this fixture.
    /// The test only needs MCC uniqueness; the production model still treats it
    /// as the category lookup key.
    pub fn mcc(&self, base: &str) -> String {
        let salt = base.bytes().fold(0_u128, |acc, byte| {
            acc.wrapping_mul(257).wrapping_add(byte as u128)
        });
        let value = ((self.uuid.as_u128() ^ salt) % 90_000_000) + 10_000_000;
        format!("{value:08}")
    }

    pub fn merchant(&self, base: &str, partner: bool) -> Merchant {
        Merchant {
            name: self.merchant_name(base),
            partner,
        }
    }

    pub fn category(&self, base: &str, cashback_rate: Decimal) -> ProductCategory {
        ProductCategory {
            mcc: self.mcc(base),
            name: self.category_name(base),
            cashback_rate,
        }
    }

    pub fn cashback(
        &self,
        customer_base: &str,
        merchant_base: &str,
        category_base: &str,
        amount: Decimal,
    ) -> CashbackRecord {
        CashbackRecord {
            customer_id: self.customer_id(customer_base),
            merchant_name: self.merchant_name(merchant_base),
            product_category: self.category_name(category_base),
            cashback_amount: amount,
        }
    }
}

pub struct TestTimer {
    kind: &'static str,
    test: String,
    fixture: String,
    started: Instant,
}

impl TestTimer {
    pub fn new(kind: &'static str, test: impl Into<String>, fixture: impl Into<String>) -> Self {
        let test = test.into();
        let fixture = fixture.into();
        info!(
            target: "cashback_rewards_rust::test_timing",
            event = "test.started",
            kind,
            test = %test,
            fixture = %fixture,
        );
        Self {
            kind,
            test,
            fixture,
            started: Instant::now(),
        }
    }
}

impl Drop for TestTimer {
    fn drop(&mut self) {
        info!(
            target: "cashback_rewards_rust::test_timing",
            event = "test.completed",
            kind = self.kind,
            test = %self.test,
            fixture = %self.fixture,
            elapsed_ms = self.started.elapsed().as_millis() as u64,
        );
    }
}

pub fn parse_purchase_timestamp() -> DateTime<Utc> {
    "2026-05-01T10:00:00Z"
        .parse()
        .expect("valid fixed purchase timestamp")
}
