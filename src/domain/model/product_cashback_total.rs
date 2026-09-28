use rust_decimal::{Decimal, RoundingStrategy};

#[derive(Debug, Clone, PartialEq)]
pub struct ProductCashbackTotal {
    pub product: String,
    pub total_cashback: Decimal,
    pub record_count: i64,
}

impl ProductCashbackTotal {
    pub fn new(product: String, total_cashback: Decimal, record_count: i64) -> Self {
        let mut total_cashback = total_cashback.round_dp_with_strategy(2, RoundingStrategy::ToZero);
        total_cashback.rescale(2);

        Self {
            product,
            total_cashback,
            record_count,
        }
    }
}
