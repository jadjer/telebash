mod auth_manager;
mod bot;
mod commands;
mod config_manager;
mod errors;
mod file_manager;
mod log_manager;
mod types;

use crate::auth_manager::AuthManager;
use crate::bot::BotManager;
use crate::config_manager::ConfigManager;
use crate::errors::BotError;
use crate::file_manager::FileManager;
use crate::log_manager::LogManager;
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    // Get config path from command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.json"
    };

    // Load configuration
    let config = ConfigManager::load_from_file(Path::new(&config_path))
        .expect(&format!("Failed to load config file: {}", config_path));

    // Initialize managers
    let auth_manager = AuthManager::new(Path::new(&config.users_file_path))
        .expect("Failed to initialize auth manager");
    let file_manager = FileManager::new()
        .expect("Failed to initialize file manager");
    let log_manager = LogManager::new(&config.log_file_path)
        .expect("Failed to initialize log manager");

    // Create and run bot
    let bot_manager = BotManager::new(&config, auth_manager, file_manager, log_manager)?;

    println!("Bot is running...");
    bot_manager.run().await?;

    Ok(())
}
