use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub telegram_token: String,
    pub auth_file_path: String,
    pub log_file_path: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedUser {
    pub user_id: u64,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub current_dir: PathBuf,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
}
