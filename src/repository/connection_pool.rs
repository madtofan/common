use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub type ServiceConnectionPool = Pool<Postgres>;
pub struct ServiceConnectionManager;

impl ServiceConnectionManager {
    pub async fn new_pool(connection_string: &str) -> anyhow::Result<ServiceConnectionPool> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(60))
            .connect(connection_string)
            .await
            .context("error while initializing the database connection pool")?;

        Ok(pool)
    }
}
