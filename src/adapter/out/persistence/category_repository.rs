use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::application::port::out::{CategoryRepository, PortError};
use crate::domain::model::ProductCategory;

use super::category_entity::ProductCategoryEntity;
use super::default_rate_entity::DefaultCashbackRateEntity;

#[derive(Clone)]
pub struct PgCategoryRepository {
    pool: PgPool,
}

impl PgCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepository {
    async fn save(&self, category: &ProductCategory) -> Result<(), PortError> {
        let entity = ProductCategoryEntity::from_domain(category);

        sqlx::query(
            r#"INSERT INTO product_category (mcc, name, cashback_rate)
               VALUES ($1, $2, $3)
               ON CONFLICT (mcc) DO UPDATE
               SET name = EXCLUDED.name, cashback_rate = EXCLUDED.cashback_rate"#,
        )
        .bind(entity.mcc)
        .bind(entity.name)
        .bind(entity.cashback_rate)
        .execute(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(())
    }

    async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError> {
        let entity = DefaultCashbackRateEntity::new(rate);

        sqlx::query(
            r#"INSERT INTO default_cashback_rate (id, rate) VALUES ($1, $2)
               ON CONFLICT (id) DO UPDATE SET rate = EXCLUDED.rate"#,
        )
        .bind(entity.id)
        .bind(entity.rate)
        .execute(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(())
    }

    async fn default_rate(&self) -> Result<Option<Decimal>, PortError> {
        let entity = sqlx::query_as::<_, DefaultCashbackRateEntity>(
            "SELECT id, rate FROM default_cashback_rate WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(entity.map(|row| row.rate))
    }

    async fn find_by_mcc(&self, mcc: &str) -> Result<Option<ProductCategory>, PortError> {
        let entity = sqlx::query_as::<_, ProductCategoryEntity>(
            "SELECT mcc, name, cashback_rate FROM product_category WHERE mcc = $1",
        )
        .bind(mcc)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(entity.map(ProductCategoryEntity::into_domain))
    }
}
