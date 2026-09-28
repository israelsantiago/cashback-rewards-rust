use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::application::port::out::{CashbackRepository, PortError};
use crate::domain::model::CashbackRecord;

use super::cashback_entity::CashbackRecordEntity;

#[derive(Clone)]
pub struct PgCashbackRepository {
    pool: PgPool,
}

impl PgCashbackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CashbackRepository for PgCashbackRepository {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError> {
        let entity = CashbackRecordEntity::from_domain(record);

        sqlx::query(
            r#"INSERT INTO cashback_record (customer_id, merchant_name, product_category, cashback_amount)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(entity.customer_id)
        .bind(entity.merchant_name)
        .bind(entity.product_category)
        .bind(entity.cashback_amount)
        .execute(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(())
    }

    async fn find_by_customer_id(
        &self,
        customer_id: &str,
    ) -> Result<Vec<CashbackRecord>, PortError> {
        let rows = sqlx::query_as::<_, CashbackRecordEntity>(
            r#"SELECT customer_id, merchant_name, product_category, cashback_amount
               FROM cashback_record
               WHERE customer_id = $1"#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(rows
            .into_iter()
            .map(CashbackRecordEntity::into_domain)
            .collect())
    }

    async fn total_for_product_category(
        &self,
        product_category: &str,
    ) -> Result<Decimal, PortError> {
        let row = sqlx::query_as::<_, (Decimal,)>(
            "SELECT COALESCE(SUM(cashback_amount), 0) FROM cashback_record WHERE product_category = $1",
        )
        .bind(product_category)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(row.0)
    }

    async fn count_for_product_category(&self, product_category: &str) -> Result<i64, PortError> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM cashback_record WHERE product_category = $1",
        )
        .bind(product_category)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(row.0)
    }
}
