use crate::config::Config;
use dashmap::DashMap;
use reqwest::Client;
use songbird::input::AuxMetadata;
use std::sync::Arc;
use uuid::Uuid;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
pub struct Data {
    pub http_client: Client,
    pub config: Arc<Config>,
    pub data: DashMap<Uuid, AuxMetadata>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            http_client: Client::new(),
            config: Arc::new(Config::default()),
            data: DashMap::new(),
        }
    }
}
