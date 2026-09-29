use rust_decimal::dec;

use crate::support::{
    acceptance::AcceptanceFixture, postgres::postgres_context, runtime::run_async,
};

#[test]
fn category_rates_cover_groceries_and_fuel() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "category-rates");
        let _timer = fixture.timer("category_rates_cover_groceries_and_fuel");

        let groceries = fixture.register_category("Groceries", dec!(0.02)).await?;
        let fuel = fixture.register_category("Fuel", dec!(0.01)).await?;
        let grocer = fixture.register_merchant("Grocer", true).await?;
        let fuel_co = fixture.register_merchant("FuelCo", true).await?;
        fixture
            .record_purchase("cust-g", &grocer, &groceries, dec!(100.00))
            .await?;
        fixture
            .record_purchase("cust-f", &fuel_co, &fuel, dec!(100.00))
            .await?;

        let g = fixture.cashback_for("cust-g").await?;
        let f = fixture.cashback_for("cust-f").await?;
        assert_eq!(g[0]["cashbackAmount"].to_string(), "2.00");
        assert_eq!(f[0]["cashbackAmount"].to_string(), "1.00");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn minimum_purchase_threshold_it() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "threshold-exact-above");
        let _timer = fixture.timer("minimum_purchase_threshold_it");

        let category = fixture.register_category("Groceries", dec!(0.02)).await?;
        let at_merchant = fixture.register_merchant("AtMart", true).await?;
        let big_merchant = fixture.register_merchant("BigMart", true).await?;
        fixture
            .record_purchase("cust-at", &at_merchant, &category, dec!(1.00))
            .await?;
        fixture
            .record_purchase("cust-big", &big_merchant, &category, dec!(25.00))
            .await?;

        let at = fixture.cashback_for("cust-at").await?;
        let big = fixture.cashback_for("cust-big").await?;
        assert_eq!(at[0]["cashbackAmount"].to_string(), "0.02");
        assert_eq!(big[0]["cashbackAmount"].to_string(), "0.50");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn threshold_uses_purchase_amount_even_for_tiny_reward() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let mut fixture = AcceptanceFixture::new(pool, "threshold-tiny-reward");
        let _timer = fixture.timer("threshold_uses_purchase_amount_even_for_tiny_reward");

        fixture.configure_default_rate(dec!(0.005)).await?;
        let merchant = fixture.register_merchant("PennySaver", true).await?;
        let unknown = fixture.identity().mcc("tiny-reward");
        fixture
            .record_purchase_with_mcc("cust-tiny", &merchant, &unknown, dec!(5.00))
            .await?;

        let records = fixture.cashback_for("cust-tiny").await?;
        assert_eq!(records[0]["cashbackAmount"].to_string(), "0.02");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn product_total_returns_zero_count_when_empty() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "product-total-empty");
        let _timer = fixture.timer("product_total_returns_zero_count_when_empty");
        let category = fixture.category_without_registration("Travel");

        let total = fixture.product_total(&category).await?;
        assert_eq!(total["product"], category.category.name);
        assert_eq!(total["totalCashback"].to_string(), "0.00");
        assert_eq!(total["recordCount"], 0);

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
