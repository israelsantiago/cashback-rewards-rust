use rust_decimal::Decimal;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(crate) struct DefaultCashbackRateEntity {
    pub(crate) id: i32,
    pub(crate) rate: Decimal,
}

impl DefaultCashbackRateEntity {
    pub(crate) const SINGLETON_ID: i32 = 1;

    pub(crate) fn new(rate: Decimal) -> Self {
        Self {
            id: Self::SINGLETON_ID,
            rate,
        }
    }
}
