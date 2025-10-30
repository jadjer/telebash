use crate::auth_manager::AuthManager;
use crate::commands::Command;
use crate::errors::BotError;
use crate::file_manager::FileManager;
use crate::log_manager::LogManager;
use crate::types::{Config, Id};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use tokio::sync::Mutex;

pub struct BotManager {
    bot: Bot,
    auth_manager: Arc<Mutex<AuthManager>>,
    file_manager: Arc<Mutex<FileManager>>,
    log_manager: Arc<LogManager>,
}

impl BotManager {
    pub fn new(
        config: &Config,
        auth_manager: AuthManager,
        file_manager: FileManager,
        log_manager: LogManager,
    ) -> Result<Self, BotError> {
        let bot = Bot::new(&config.telegram_token);
        let _ = bot.set_my_commands(Command::bot_commands());

        Ok(BotManager {
            bot,
            auth_manager: Arc::new(Mutex::new(auth_manager)),
            file_manager: Arc::new(Mutex::new(file_manager)),
            log_manager: Arc::new(log_manager),
        })
    }

    pub async fn run(&self) -> Result<(), BotError> {
        let handler = Update::filter_message().branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(Self::handle_command),
        );

        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![
                self.auth_manager.clone(),
                self.file_manager.clone(),
                self.log_manager.clone()
            ])
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    async fn handle_command(
        bot: Bot,
        msg: Message,
        cmd: Command,
        auth_manager: Arc<Mutex<AuthManager>>,
        file_manager: Arc<Mutex<FileManager>>,
        log_manager: Arc<LogManager>,
    ) -> Result<(), BotError> {
        if let Some(user) = &msg.from {
            let user_id = user.id.0;

            match cmd {
                Command::Help => {
                    Self::handle_help(bot, msg, user_id, &auth_manager).await?;
                }
                Command::AuthRequest => {
                    Self::handle_auth(bot, msg,  user_id, auth_manager, log_manager).await?;
                }
                Command::Auth(code) => {
                    Self::handle_auth_code(bot, msg, code, user_id, auth_manager, log_manager).await?;
                }
                _ => {
                    if auth_manager.lock().await.is_authorized(user_id) {
                        match cmd {
                            Command::Ls => {
                                Self::handle_ls(bot, msg, user_id, file_manager).await?;
                            }
                            Command::Cd(path) => {
                                Self::handle_cd(bot, msg, path, user_id, file_manager).await?;
                            }
                            Command::Download(filename) => {
                                Self::handle_download(bot, msg, filename, user_id, file_manager).await?;
                            }
                            Command::Exec(command) => {
                                Self::handle_exec(bot, msg, command, user_id, file_manager).await?;
                            }
                            Command::Pwd => {
                                Self::handle_pwd(bot, msg, user_id, file_manager).await?;
                            }
                            _ => {}
                        }
                    } else {
                        bot.send_message(msg.chat.id, "❌ Unauthorized. Use /auth to get access.")
                            .await
                            .map_err(|e| BotError::TelegramError(e.to_string()))?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_help(
        bot: Bot,
        msg: Message,
        user_id: Id,
        auth_manager: &Arc<Mutex<AuthManager>>,
    ) -> Result<(), BotError> {
        let is_authorized = auth_manager.lock().await.is_authorized(user_id);

        let help_text = if is_authorized {
            "Available commands:\n\
            /help - Show this help\n\
            /ls - List directory contents\n\
            /cd <directory> - Change directory\n\
            /download <filename> - Download file\n\
            /exec <command> - Execute command\n\
            /pwd - Print working directory"
        } else {
            "Available commands:\n\
            /help - Show this help\n\
            /auth - Authorize with access code"
        };

        bot.send_message(msg.chat.id, help_text)
            .await
            .map_err(|e| BotError::TelegramError(e.to_string()))?;

        Ok(())
    }

    async fn handle_auth(
        bot: teloxide::Bot,
        msg: Message,
        user_id: Id,
        auth_manager: Arc<Mutex<AuthManager>>,
        log_manager: Arc<LogManager>,
    ) -> Result<(), BotError> {
        let mut auth_manager = auth_manager.lock().await;

        if auth_manager.is_authorized(user_id) {
            bot.send_message(msg.chat.id, "✅ You are already authorized.")
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            return Ok(());
        }

        let access_code = auth_manager.generate_access_code(user_id);

        log_manager.log(
            log::Level::Info,
            &format!(
                "Access code generated for user {}: {}",
                user_id, access_code
            ),
        )?;

        println!("Access code for user {}: {}", user_id, access_code);

        bot.send_message(
            msg.chat.id,
            "🔑 Please enter the access code displayed in the console.",
        )
        .await
        .map_err(|e| BotError::TelegramError(e.to_string()))?;

        Ok(())
    }

    async fn handle_auth_code(
        bot: Bot,
        msg: Message,
        code: String,
        user_id: Id,
        auth_manager: Arc<Mutex<AuthManager>>,
        log_manager: Arc<LogManager>,
    ) -> Result<(), BotError> {
        let mut auth_manager = auth_manager.lock().await;

        if auth_manager.is_authorized(user_id) {
            bot.send_message(msg.chat.id, "✅ You are already authorized.")
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            return Ok(());
        }

        let is_verified = match auth_manager.verify_access_code(&code, user_id) {
            Ok(_) => {}
            Err(_) => {}
        };

        // log_manager.log(
        //     log::Level::Info,
        //     &format!(
        //         "Access code generated for user {}: {}",
        //         user_id, access_code
        //     ),
        // )?;
        //
        // println!("Access code for user {}: {}", user_id, access_code);
        //
        // bot.send_message(
        //     msg.chat.id,
        //     "🔑 Please enter the access code displayed in the console.",
        // )
        //     .await
        //     .map_err(|e| BotError::TelegramError(e.to_string()))?;

        Ok(())
    }

    fn escape_text(text: &str) -> String {
        text.replace('.', "\\.")
            .replace('!', "\\!")
            .replace('-', "\\-")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('+', "\\+")
            .replace('=', "\\=")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('>', "\\>")
            .replace('#', "\\#")
    }

    async fn handle_ls(
        bot: Bot,
        msg: Message,
        user_id: Id,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Result<(), BotError> {
        let file_manager = file_manager.lock().await;
        let items = file_manager.list_directory(user_id)?;

        if items.is_empty() {
            bot.send_message(msg.chat.id, "📁 Directory is empty")
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            return Ok(());
        }

        // let mut response = String::new();
        // response.push_str("📁 Directory contents:\n\n");
        //
        // for item in items {
        //     let icon = if item.is_directory { "📁" } else { "📄" };
        //     let command = if item.is_directory {
        //         format!("cd {}", item.name)
        //     } else {
        //         format!("download {}", item.name)
        //     };
        //
        //     response.push_str(&format!("{} {} `/{}`\n", icon, item.name, command));
        // }
        //
        // response = Self::escape_text(response.as_str());
        //
        // bot.send_message(msg.chat.id, response)
        //     .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        //     .await
        //     .map_err(|e| BotError::TelegramError(e.to_string()))?;

        let mut response = String::new();
        let mut keyboard = Vec::new();
        let mut current_row = Vec::new();

        for item in items {
            let icon = if item.is_directory { "📁" } else { "📄" };
            let line = format!("{} {}\n", icon, item.name);
            response.push_str(&line);

            let button_text = if item.is_directory {
                format!("📁 {}", item.name)
            } else {
                format!("📄 {}", item.name)
            };

            let callback_data = if item.is_directory {
                format!("/cd {}", item.name)
            } else {
                format!("/download {}", item.name)
            };

            current_row.push(InlineKeyboardButton::callback(button_text, callback_data));

            if current_row.len() == 2 {
                keyboard.push(current_row);
                current_row = Vec::new();
            }
        }

        if !current_row.is_empty() {
            keyboard.push(current_row);
        }

        let current_directory = file_manager.get_current_directory(user_id);

        // Add navigation buttons
        if current_directory.parent().is_some() {
            keyboard.push(vec![InlineKeyboardButton::callback(
                "⬆️ ..",
                "/cd ..".to_string(),
            )]);
        }

        let reply_markup = InlineKeyboardMarkup::new(keyboard);

        bot
            .send_message(msg.chat.id, response)
            .reply_markup(reply_markup)
            .send()
            .await
            .map_err(|e| {
                BotError::TelegramError(format!("Failed to send message: {}", e))
            })?;

        Ok(())
    }

    async fn handle_cd(
        bot: Bot,
        msg: Message,
        path: String,
        user_id: Id,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Result<(), BotError> {
        let mut file_manager = file_manager.lock().await;

        match file_manager.change_directory(user_id, &path) {
            Ok(()) => {
                let current_dir = file_manager.get_current_directory(user_id);
                bot.send_message(
                    msg.chat.id,
                    format!("📁 Changed directory to: {}", current_dir.display()),
                )
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("❌ Error: {}", e))
                    .await
                    .map_err(|e| BotError::TelegramError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn handle_download(
        bot: Bot,
        msg: Message,
        filename: String,
        user_id: Id,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Result<(), BotError> {
        let file_manager = file_manager.lock().await;

        if !file_manager.file_exists(user_id, &filename) {
            bot.send_message(msg.chat.id, "❌ File not found")
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            return Ok(());
        }

        if !file_manager.is_file(user_id, &filename) {
            bot.send_message(msg.chat.id, "❌ Cannot download directories")
                .await
                .map_err(|e| BotError::TelegramError(e.to_string()))?;
            return Ok(());
        }

        let file_path = file_manager.get_file_path(user_id, &filename);

        bot.send_document(msg.chat.id, teloxide::types::InputFile::file(&file_path))
            .await
            .map_err(|e| BotError::TelegramError(e.to_string()))?;

        Ok(())
    }

    async fn handle_exec(
        bot: Bot,
        msg: Message,
        command: String,
        user_id: Id,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Result<(), BotError> {
        let file_manager = file_manager.lock().await;
        let current_dir = file_manager.get_current_directory(user_id);

        // Basic command execution - in production, you'd want more security
        let output = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(["/C", &command])
                .current_dir(current_dir)
                .output()
        } else {
            std::process::Command::new("sh")
                .args(["-c", &command])
                .current_dir(current_dir)
                .output()
        };

        match output {
            Ok(output) => {
                let response = if output.status.success() {
                    format!(
                        "✅ Command executed successfully:\n```\n{}\n```",
                        String::from_utf8_lossy(&output.stdout)
                    )
                } else {
                    format!(
                        "❌ Command failed:\n```\n{}\n```",
                        String::from_utf8_lossy(&output.stderr)
                    )
                };

                bot.send_message(msg.chat.id, response)
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await
                    .map_err(|e| BotError::TelegramError(e.to_string()))?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("❌ Failed to execute command: {}", e))
                    .await
                    .map_err(|e| BotError::TelegramError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn handle_pwd(
        bot: Bot,
        msg: Message,
        user_id: Id,
        file_manager: Arc<Mutex<FileManager>>,
    ) -> Result<(), BotError> {
        let file_manager = file_manager.lock().await;
        let current_dir = file_manager.get_current_directory(user_id);

        bot.send_message(
            msg.chat.id,
            format!("📁 Current directory: {}", current_dir.display()),
        )
        .await
        .map_err(|e| BotError::TelegramError(e.to_string()))?;

        Ok(())
    }
}
