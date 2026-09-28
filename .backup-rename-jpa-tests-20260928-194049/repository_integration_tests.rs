use cashback_rewards_rust::adapter::out::persistence::{
    PgCashbackRepository, PgCategoryRepository, PgMerchantRepository,
};
use cashback_rewards_rust::application::port::out::{
    CashbackRepository, CategoryRepository, MerchantRepository,
};
use cashback_rewards_rust::domain::model::{CashbackRecord, Merchant, ProductCategory};
use rust_decimal::dec;

#[tokio::test]
async fn merchant_repository_matches_java_jpa_adapter_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let context = crate::support::postgres_context().await?;
    let repository = PgMerchantRepository::new(context.pool);

    let merchant = Merchant {
        name: "GreenGrocer".into(),
        partner: true,
    };
    repository.save(&merchant).await?;

    assert_eq!(
        repository.find_by_name("GreenGrocer").await?,
        Some(merchant.clone())
    );
    assert_eq!(
        repository.find_by_name("greengrocer").await?,
        Some(merchant.clone())
    );
    assert_eq!(
        repository.find_by_name("  GREENGROCER  ").await?,
        Some(merchant)
    );
    assert_eq!(repository.find_by_name("Missing").await?, None);

    Ok(())
}

#[tokio::test]
async fn category_repository_matches_java_jpa_adapter_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let context = crate::support::postgres_context().await?;
    let repository = PgCategoryRepository::new(context.pool);

    assert_eq!(repository.find_by_mcc("9999").await?, None);

    let category = ProductCategory {
        mcc: "5411".into(),
        name: "Groceries".into(),
        cashback_rate: dec!(0.02),
    };
    repository.save(&category).await?;
    assert_eq!(
        repository.find_by_mcc("5411").await?,
        Some(category.clone())
    );

    repository.save_default_rate(dec!(0.005)).await?;
    assert_eq!(repository.default_rate().await?, Some(dec!(0.005)));

    repository.save_default_rate(dec!(0.01)).await?;
    assert_eq!(repository.default_rate().await?, Some(dec!(0.01)));

    Ok(())
}

#[tokio::test]
async fn cashback_repository_matches_java_jpa_adapter_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let context = crate::support::postgres_context().await?;
    let repository = PgCashbackRepository::new(context.pool);

    repository
        .save(&CashbackRecord {
            customer_id: "cust-001".into(),
            merchant_name: "Market-A".into(),
            product_category: "Groceries".into(),
            cashback_amount: dec!(2.40),
        })
        .await?;
    repository
        .save(&CashbackRecord {
            customer_id: "cust-001".into(),
            merchant_name: "FuelCo".into(),
            product_category: "Fuel".into(),
            cashback_amount: dec!(1.00),
        })
        .await?;
    repository
        .save(&CashbackRecord {
            customer_id: "cust-999".into(),
            merchant_name: "OtherShop".into(),
            product_category: "Other".into(),
            cashback_amount: dec!(0.50),
        })
        .await?;
    repository
        .save(&CashbackRecord {
            customer_id: "cust-002".into(),
            merchant_name: "Market-B".into(),
            product_category: "Groceries".into(),
            cashback_amount: dec!(1.60),
        })
        .await?;
    repository
        .save(&CashbackRecord {
            customer_id: "cust-003".into(),
            merchant_name: "FuelCo".into(),
            product_category: "Fuel".into(),
            cashback_amount: dec!(5.00),
        })
        .await?;

    let customer_records = repository.find_by_customer_id("cust-001").await?;
    assert_eq!(customer_records.len(), 2);
    assert!(
        customer_records
            .iter()
            .any(|r| r.merchant_name == "Market-A")
    );
    assert!(customer_records.iter().any(|r| r.merchant_name == "FuelCo"));

    assert_eq!(
        repository.total_for_product_category("Groceries").await?,
        dec!(4.00)
    );
    assert_eq!(
        repository.total_for_product_category("Travel").await?,
        dec!(0.00)
    );
    assert_eq!(repository.count_for_product_category("Groceries").await?, 2);

    Ok(())
}
