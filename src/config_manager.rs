use crate::errors::BotError;
use crate::types::Config;
use std::fs;

pub struct ConfigManager;

impl ConfigManager {
    pub fn load_config(path: &str) -> Result<Config, BotError> {
        let config_content = fs::read_to_string(path)
            .map_err(|e| BotError::ConfigError(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&config_content)
            .map_err(|e| BotError::ConfigError(format!("Failed to parse config: {}", e)))
    }
}