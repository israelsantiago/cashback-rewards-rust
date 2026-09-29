use rust_decimal::dec;
use serde_json::json;

use crate::support::{
    acceptance::AcceptanceFixture, postgres::postgres_context, runtime::run_async,
};

#[test]
fn postgres_acceptance_covers_cashback_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;

        // Each scenario is independently namespaced. No TRUNCATE is used.
        {
            let fixture = AcceptanceFixture::new(pool.clone(), "basic-partner");
            let _timer = fixture.timer("basic_partner_cashback");
            let category = fixture.register_category("Groceries", dec!(0.02)).await?;
            let merchant = fixture.register_merchant("GreenGrocer", true).await?;
            fixture
                .record_purchase("cust-001", &merchant, &category, dec!(80.00))
                .await?;
            let records = fixture.cashback_for("cust-001").await?;
            assert_eq!(records.as_array().map(Vec::len), Some(1));
            assert_eq!(records[0]["cashbackAmount"].to_string(), "1.60");
        }

        {
            let fixture = AcceptanceFixture::new(pool.clone(), "basic-non-partner");
            let _timer = fixture.timer("basic_non_partner_no_cashback");
            let category = fixture.register_category("Groceries", dec!(0.02)).await?;
            let merchant = fixture.register_merchant("CornerCafe", false).await?;
            fixture
                .record_purchase("cust-002", &merchant, &category, dec!(80.00))
                .await?;
            assert_eq!(fixture.cashback_for("cust-002").await?, json!([]));
        }

        {
            let mut fixture = AcceptanceFixture::new(pool.clone(), "unmapped-default-rate");
            let _timer = fixture.timer("unmapped_mcc_uses_default_rate");
            fixture.configure_default_rate(dec!(0.005)).await?;
            let merchant = fixture.register_merchant("CityPharmacy", true).await?;
            let unknown = fixture.identity().mcc("unmapped");
            fixture
                .record_purchase_with_mcc("cust-default", &merchant, &unknown, dec!(100.00))
                .await?;
            let records = fixture.cashback_for("cust-default").await?;
            assert_eq!(records[0]["productCategory"], "Other");
            assert_eq!(records[0]["cashbackAmount"].to_string(), "0.50");
        }

        {
            let fixture = AcceptanceFixture::new(pool.clone(), "minimum-threshold");
            let _timer = fixture.timer("below_threshold_earns_nothing");
            let category = fixture.register_category("Groceries", dec!(0.02)).await?;
            let merchant = fixture.register_merchant("TinyGrocer", true).await?;
            fixture
                .record_purchase("cust-threshold", &merchant, &category, dec!(0.99))
                .await?;
            assert_eq!(fixture.cashback_for("cust-threshold").await?, json!([]));
        }

        {
            let fixture = AcceptanceFixture::new(pool.clone(), "product-total");
            let _timer = fixture.timer("aggregate_across_customers");
            let category = fixture.register_category("Groceries", dec!(0.02)).await?;
            let merchant_a = fixture.register_merchant("Market-A", true).await?;
            let merchant_b = fixture.register_merchant("Market-B", true).await?;
            fixture
                .record_purchase("cust-a", &merchant_a, &category, dec!(120.00))
                .await?;
            fixture
                .record_purchase("cust-b", &merchant_b, &category, dec!(80.00))
                .await?;
            let total = fixture.product_total(&category).await?;
            assert_eq!(total["totalCashback"].to_string(), "4.00");
            assert_eq!(total["recordCount"], 2);
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
