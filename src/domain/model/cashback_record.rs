use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct CashbackRecord {
    pub customer_id: String,
    pub merchant_name: String,
    pub product_category: String,
    pub cashback_amount: Decimal,
}
