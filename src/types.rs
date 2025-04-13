use crate::config::Config;
use dashmap::DashMap;
use reqwest::Client;
use songbird::input::AuxMetadata;
use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;
use uuid::Uuid;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type DbPool = Pool<Postgres>;

#[allow(dead_code)]
pub struct Data {
    pub http_client: Client,
    pub config: Arc<Config>,
    pub data: DashMap<Uuid, AuxMetadata>,
    pub db_pool: DbPool,
}

impl Data {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let db_pool = crate::db::init_db_pool(&config.db_url, config.db_pool_size).await?;
        Ok(Self {
            http_client: Client::new(),
            config,
            data: DashMap::new(),
            db_pool,
        })
    }
}

impl Default for Data {
    fn default() -> Self {
        // For default usage only (synchronous version); consider using Data::new in async contexts.
        unimplemented!("Use Data::new() in an async context")
    }
}
