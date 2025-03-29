use crate::config::Config;
use reqwest::Client;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[allow(dead_code)]
pub struct Data {
    pub http_client: Client,
    pub config: Arc<Config>,
}

