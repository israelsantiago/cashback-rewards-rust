#[cfg(feature = "postgres-acceptance")]
#[path = "support/mod.rs"]
pub mod support;

#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/basic_cashback_calculation_it.rs"]
mod basic_cashback_calculation_it;

#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/merchant_categories_and_eligibility_it.rs"]
mod merchant_categories_and_eligibility_it;

#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/minimum_purchase_threshold_it.rs"]
mod minimum_purchase_threshold_it;

#[cfg(feature = "postgres-acceptance")]
#[path = "acceptance/total_cashback_per_product_it.rs"]
mod total_cashback_per_product_it;
