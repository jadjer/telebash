use std::fmt;

#[derive(Debug)]
pub enum BotError {
    ConfigError(String),
    AuthError(String),
    FileError(String),
    LogError(String),
    TelegramError(String),
    SerializationError(String),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            BotError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            BotError::FileError(msg) => write!(f, "File error: {}", msg),
            BotError::LogError(msg) => write!(f, "Log error: {}", msg),
            BotError::TelegramError(msg) => write!(f, "Telegram error: {}", msg),
            BotError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for BotError {}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> Self {
        BotError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for BotError {
    fn from(err: std::io::Error) -> Self {
        BotError::FileError(err.to_string())
    }
}