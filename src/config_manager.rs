use crate::errors::BotError;
use crate::types::Config;
use std::fs;
use std::path::Path;

pub struct ConfigManager;

impl ConfigManager {
    pub fn load_from_file(path: &Path) -> Result<Config, BotError> {
        let config_content = fs::read_to_string(path)
            .map_err(|e| BotError::ConfigError(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&config_content)
            .map_err(|e| BotError::ConfigError(format!("Failed to parse config: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_load_valid_config() {
        let json_content = r#"
        {
            "telegram_token": "123qwe456asd",
            "users_file_path": "users.json",
            "log_file_path": "logs.json"
        }
        "#;

        let temp_file = create_temp_json(json_content);
        let config = ConfigManager::load_from_file(temp_file.path()).unwrap();

        assert_eq!(config.telegram_token, "123qwe456asd");
        assert_eq!(config.users_file_path, "users.json");
        assert_eq!(config.log_file_path, "logs.json");
    }

    #[test]
    fn test_load_invalid_json() {
        let invalid_json = r#"
        {
            "telegram_token": "123qwe456asd",
            "users_file_path": "users.json",
            "log_file_path": "logs.json",
        }
        "#;

        let temp_file = create_temp_json(invalid_json);
        let result = ConfigManager::load_from_file(temp_file.path());

        assert!(result.is_err());
        // assert!(result.unwrap_err().contains("parse JSON"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = ConfigManager::load_from_file(Path::new("nonexistent_file.json"));
        assert!(result.is_err());
        // assert!(result.unwrap_err().contains("read file"));
    }
}