use crate::{config::Config, db};
use dashmap::DashMap;
use reqwest::Client;
use songbird::input::AuxMetadata;
use spotify_rs::{
    ClientCredsClient, ClientCredsFlow,
    auth::{NoVerifier, Token},
    client::Client as SpotifyClient,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type DbPool = Pool<Postgres>;

pub struct Data {
    pub http_client: Client,
    pub config: Arc<Config>,
    pub data: DashMap<Uuid, AuxMetadata>,
    pub database: db::Database,
    pub spotify_client: Mutex<SpotifyClient<Token, ClientCredsFlow, NoVerifier>>,
}

impl Data {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let database = crate::db::Database::new(&config.db_url, config.db_pool_size).await?;

        let auth_flow =
            ClientCredsFlow::new(&config.spotify_client_id, &config.spotify_client_secret);
        let spotify_client = ClientCredsClient::authenticate(auth_flow).await?;

        Ok(Self {
            http_client: Client::new(),
            config,
            data: DashMap::new(),
            database,
            spotify_client: Mutex::new(spotify_client),
        })
    }
}
