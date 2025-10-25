use crate::errors::BotError;
use log::{LevelFilter, Record};
use simple_logger::SimpleLogger;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct LogManager {
    file: Mutex<std::fs::File>,
}

impl LogManager {
    pub fn new(log_file_path: &str) -> Result<Self, BotError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path)
            .map_err(|e| BotError::LogError(format!("Failed to open log file: {}", e)))?;

        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .map_err(|e| BotError::LogError(format!("Failed to initialize logger: {}", e)))?;

        Ok(LogManager {
            file: Mutex::new(file),
        })
    }

    pub fn log(&self, level: log::Level, message: &str) -> Result<(), BotError> {
        let log_entry = format!("[{}] {}\n", level, message);

        let mut file_guard = self.file.lock()
            .map_err(|e| BotError::LogError(format!("Failed to lock log file: {}", e)))?;

        file_guard.write_all(log_entry.as_bytes())
            .map_err(|e| BotError::LogError(format!("Failed to write to log file: {}", e)))?;

        file_guard.flush()
            .map_err(|e| BotError::LogError(format!("Failed to flush log file: {}", e)))?;

        Ok(())
    }
}