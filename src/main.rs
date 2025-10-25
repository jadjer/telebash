mod errors;
mod commands;
mod types;
mod bot;
mod auth_manager;
mod log_manager;
mod file_manager;
mod config_manager;

use crate::config_manager::ConfigManager;
use crate::bot::BotManager;
use crate::auth_manager::AuthManager;
use crate::file_manager::FileManager;
use crate::log_manager::LogManager;
use crate::errors::BotError;
use std::env;

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
    let config = ConfigManager::load_config(config_path)?;

    // Initialize managers
    let auth_manager = AuthManager::new(&config.auth_file_path)?;
    let file_manager = FileManager::new(&config.working_directory)?;
    let log_manager = LogManager::new(&config.log_file_path)?;

    // Log startup
    log_manager.log(
        log::Level::Info,
        &format!("Bot started with working directory: {}", config.working_directory),
    )?;

    // Create and run bot
    let bot_manager = BotManager::new(&config, auth_manager, file_manager, log_manager)?;

    println!("Bot is running...");
    bot_manager.run().await?;

    Ok(())
}