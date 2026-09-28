use rust_decimal::Decimal;

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
