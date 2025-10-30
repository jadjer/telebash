use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Id = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub telegram_token: String,
    pub users_file_path: String,
    pub log_file_path: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedUsers {
    pub users: HashSet<Id>,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
}
