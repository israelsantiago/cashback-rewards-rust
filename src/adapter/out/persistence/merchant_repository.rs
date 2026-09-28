use async_trait::async_trait;
use sqlx::PgPool;

use crate::application::port::out::{MerchantRepository, PortError};
use crate::domain::model::Merchant;

use super::merchant_entity::{MerchantEntity, normalize};

#[derive(Clone)]
pub struct PgMerchantRepository {
    pool: PgPool,
}

impl PgMerchantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MerchantRepository for PgMerchantRepository {
    async fn save(&self, merchant: &Merchant) -> Result<(), PortError> {
        let entity = MerchantEntity::from_domain(merchant);

        sqlx::query(r#"INSERT INTO merchant (normalized_name, name, partner) VALUES ($1, $2, $3)"#)
            .bind(entity.normalized_name)
            .bind(entity.name)
            .bind(entity.partner)
            .execute(&self.pool)
            .await
            .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Merchant>, PortError> {
        let entity = sqlx::query_as::<_, MerchantEntity>(
            "SELECT normalized_name, name, partner FROM merchant WHERE normalized_name = $1",
        )
        .bind(normalize(name))
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| PortError::Database(error.to_string()))?;

        Ok(entity.map(MerchantEntity::into_domain))
    }
}
