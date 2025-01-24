use kdl::{KdlDocument, KdlValue};
use poise::serenity_prelude as serenity;
use serenity::model::user::OnlineStatus;
use std::fs;
use std::path::Path;
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
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self, LoadConfigError> {
        let contents = fs::read_to_string(config_path)?;
        let kdl_doc: KdlDocument = contents.parse()?;
        Self::parse_config(&kdl_doc)
    }

    fn parse_config(kdl_doc: &KdlDocument) -> Result<Self, LoadConfigError> {
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

        Ok(Config { status })
    }
}
