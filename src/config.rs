use kdl::{KdlDocument, KdlValue};
use poise::serenity_prelude as serenity;
use serenity::model::user::OnlineStatus;
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Kdl(#[from] kdl::KdlError),
    #[error("Missing or invalid configuration data")]
    InvalidConfig,
}

#[derive(Clone)]
pub struct Config {
    pub status: OnlineStatus,
    pub db_url: String,
    pub db_pool_size: u32,
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self, LoadConfigError> {
        let contents = fs::read_to_string(config_path)?;
        let kdl_doc: KdlDocument = contents.parse()?;
        Self::parse_config(&kdl_doc)
    }

    fn parse_config(kdl_doc: &KdlDocument) -> Result<Self, LoadConfigError> {
        // Get misc node for bot status
        let misc_node = kdl_doc.get("misc").ok_or(LoadConfigError::InvalidConfig)?;
        let status_value = misc_node
            .get("status")
            .and_then(KdlValue::as_string)
            .ok_or(LoadConfigError::InvalidConfig)?;

        let status = match status_value {
            "do-not-disturb" => OnlineStatus::DoNotDisturb,
            "idle" => OnlineStatus::Idle,
            "invisible" => OnlineStatus::Invisible,
            "online" => OnlineStatus::Online,
            _ => return Err(LoadConfigError::InvalidConfig),
        };

        // Get database node for connection details
        let db_node = kdl_doc
            .get("database")
            .ok_or(LoadConfigError::InvalidConfig)?;
        let db_url = db_node
            .get("url")
            .and_then(KdlValue::as_string)
            .ok_or(LoadConfigError::InvalidConfig)?
            .to_string();
        let db_pool_size = db_node
            .get("pool_size")
            .and_then(KdlValue::as_integer)
            .ok_or(LoadConfigError::InvalidConfig)? as u32;

        Ok(Config {
            status,
            db_url,
            db_pool_size,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            status: OnlineStatus::Online,
            db_url: "postgres://user:password@localhost/winmusic".to_string(),
            db_pool_size: 5,
        }
    }
}
