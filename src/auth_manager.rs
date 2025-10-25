use crate::errors::BotError;
use crate::types::{AuthorizedUsers, UserInfo};
use rand::Rng;
use rand::distr::{Alphanumeric, Uniform, StandardUniform};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthManager {
    authorized_users: AuthorizedUsers,
    auth_file_path: String,
    access_codes: HashMap<String, i64>, // code -> user_id
}

impl AuthManager {
    pub fn new(auth_file_path: &str) -> Result<Self, BotError> {
        let authorized_users = Self::load_authorized_users(auth_file_path)?;

        Ok(AuthManager {
            authorized_users,
            auth_file_path: auth_file_path.to_string(),
            access_codes: HashMap::new(),
        })
    }

    fn load_authorized_users(path: &str) -> Result<AuthorizedUsers, BotError> {
        match fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content)
                    .map_err(|e| BotError::AuthError(format!("Failed to parse auth file: {}", e)))
            }
            Err(_) => Ok(AuthorizedUsers { users: HashMap::new() }),
        }
    }

    fn save_authorized_users(&self) -> Result<(), BotError> {
        let content = serde_json::to_string_pretty(&self.authorized_users)
            .map_err(|e| BotError::SerializationError(e.to_string()))?;

        fs::write(&self.auth_file_path, content)
            .map_err(|e| BotError::AuthError(format!("Failed to save auth file: {}", e)))?;

        Ok(())
    }

    pub fn generate_access_code(&mut self, user_id: i64) -> String {
        let mut rng = rand::rng();
        let random_number = rng.random_range(100000..=999999);
        let code = random_number.to_string();

        self.access_codes.insert(code.clone(), user_id);
        code
    }

    pub fn verify_access_code(&mut self, code: &str, user_id: i64, username: Option<String>) -> Result<bool, BotError> {
        if let Some(stored_user_id) = self.access_codes.get(code) {
            if *stored_user_id == user_id {
                let user_info = UserInfo {
                    user_id,
                    username,
                    authorized_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| BotError::AuthError(e.to_string()))?
                        .as_secs()
                        .to_string(),
                };

                self.authorized_users.users.insert(user_id, user_info);
                self.access_codes.remove(code);
                self.save_authorized_users()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_authorized(&self, user_id: i64) -> bool {
        self.authorized_users.users.contains_key(&user_id)
    }

    pub fn get_authorized_users(&self) -> &HashMap<i64, UserInfo> {
        &self.authorized_users.users
    }
}