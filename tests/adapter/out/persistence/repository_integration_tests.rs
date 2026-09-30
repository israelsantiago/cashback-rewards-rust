use cashback_rewards_rust::adapter::out::persistence::{
    PgCashbackRepository, PgCategoryRepository, PgMerchantRepository,
};
use cashback_rewards_rust::application::port::out::{
    CashbackRepository, CategoryRepository, MerchantRepository,
};
use rust_decimal::dec;

use crate::support::{
    fixtures::{FixtureIdentity, TestTimer},
    postgres::{default_rate_guard, postgres_context},
    runtime::run_async,
};

#[test]
fn merchant_repository_test() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = FixtureIdentity::new("merchant-repository");
        let _timer = TestTimer::new("repository", "merchant_repository_test", fixture.id());
        let repository = PgMerchantRepository::new(pool);
        let merchant = fixture.merchant("GreenGrocer", true);

        repository.save(&merchant).await?;

        assert_eq!(
            repository.find_by_name(&merchant.name).await?,
            Some(merchant.clone())
        );
        assert_eq!(
            repository
                .find_by_name(&merchant.name.to_lowercase())
                .await?,
            Some(merchant.clone())
        );
        assert_eq!(
            repository
                .find_by_name(&format!("  {}  ", merchant.name))
                .await?,
            Some(merchant)
        );
        assert_eq!(repository.find_by_name("Missing").await?, None);

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn category_repository_test() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = FixtureIdentity::new("category-repository");
        let _timer = TestTimer::new("repository", "category_repository_test", fixture.id());
        let _rate_guard = default_rate_guard().await;
        let repository = PgCategoryRepository::new(pool);
        let category = fixture.category("Groceries", dec!(0.02));

        assert_eq!(repository.find_by_mcc(&fixture.mcc("unknown")).await?, None);
        repository.save(&category).await?;
        assert_eq!(
            repository.find_by_mcc(&category.mcc).await?,
            Some(category.clone())
        );

        repository.save_default_rate(dec!(0.005)).await?;
        assert_eq!(repository.default_rate().await?, Some(dec!(0.005)));
        repository.save_default_rate(dec!(0.01)).await?;
        assert_eq!(repository.default_rate().await?, Some(dec!(0.01)));

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn cashback_repository_test() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = FixtureIdentity::new("cashback-repository");
        let _timer = TestTimer::new("repository", "cashback_repository_test", fixture.id());
        let repository = PgCashbackRepository::new(pool);

        let records = [
            fixture.cashback("cust-001", "Market-A", "Groceries", dec!(2.40)),
            fixture.cashback("cust-001", "FuelCo", "Fuel", dec!(1.00)),
            fixture.cashback("cust-999", "OtherShop", "Other", dec!(0.50)),
            fixture.cashback("cust-002", "Market-B", "Groceries", dec!(1.60)),
            fixture.cashback("cust-003", "FuelCo", "Fuel", dec!(5.00)),
        ];

        for record in &records {
            repository.save(record).await?;
        }

        let customer_id = fixture.customer_id("cust-001");
        let customer_records = repository.find_by_customer_id(&customer_id).await?;
        assert_eq!(customer_records.len(), 2);
        let saved_customer_record = customer_records
            .iter()
            .find(|r| r.merchant_name == fixture.merchant_name("Market-A"))
            .expect("expected fixture Market-A record");
        assert_eq!(saved_customer_record.customer_id, customer_id);
        assert_eq!(
            saved_customer_record.merchant_name,
            fixture.merchant_name("Market-A")
        );
        assert_eq!(
            saved_customer_record.product_category,
            fixture.category_name("Groceries")
        );
        assert_eq!(saved_customer_record.cashback_amount, dec!(2.40));

        assert!(
            customer_records
                .iter()
                .any(|r| r.merchant_name == fixture.merchant_name("FuelCo"))
        );

        let groceries = fixture.category_name("Groceries");
        let travel = fixture.category_name("Travel");
        assert_eq!(
            repository.total_for_product_category(&groceries).await?,
            dec!(4.00)
        );
        assert_eq!(
            repository.total_for_product_category(&travel).await?,
            dec!(0.00)
        );
        assert_eq!(repository.count_for_product_category(&groceries).await?, 2);

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
