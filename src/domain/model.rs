use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct Merchant {
    pub name: String,
    pub partner: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductCategory {
    pub mcc: String,
    pub name: String,
    pub cashback_rate: Decimal,
}

impl ProductCategory {
    pub fn unmapped(mcc: impl Into<String>, default_rate: Decimal) -> Self {
        Self {
            mcc: mcc.into(),
            name: "Other".to_string(),
            cashback_rate: default_rate,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CashbackRecord {
    pub customer_id: String,
    pub merchant_name: String,
    pub product_category: String,
    pub cashback_amount: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductCashbackTotal {
    pub product: String,
    pub total_cashback: Decimal,
    pub record_count: i64,
}

impl ProductCashbackTotal {
    pub fn new(product: String, total_cashback: Decimal, record_count: i64) -> Self {
        Self {
            product,
            total_cashback: total_cashback
                .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::ToZero),
            record_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimumPurchaseThreshold(pub Decimal);

impl Default for MinimumPurchaseThreshold {
    fn default() -> Self {
        Self(Decimal::new(100, 2))
    }
}

impl MinimumPurchaseThreshold {
    pub fn is_met_by(&self, amount: Decimal) -> bool {
        amount >= self.0
    }
}
