use crate::errors::BotError;
use crate::types::{AuthorizedUsers, Id};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub struct AuthManager {
    authorized_users: AuthorizedUsers,
    users_file_path: PathBuf,
    access_codes: HashMap<String, u64>, // code -> user_id
}

impl AuthManager {
    pub fn new(file_path: &Path) -> Result<Self, BotError> {
        let authorized_users = Self::load_authorized_users(file_path)?;

        Ok(AuthManager {
            authorized_users,
            users_file_path: file_path.to_path_buf(),
            access_codes: HashMap::new(),
        })
    }

    fn load_authorized_users(file_path: &Path) -> Result<AuthorizedUsers, BotError> {
        match fs::read_to_string(file_path) {
            Ok(content) => serde_json::from_str(&content)
                .map_err(|e| BotError::AuthError(format!("Failed to parse auth file: {}", e))),
            Err(_) => Ok(AuthorizedUsers {
                users: HashSet::new(),
            }),
        }
    }

    fn save_authorized_users(&self) -> Result<(), BotError> {
        let content = serde_json::to_string_pretty(&self.authorized_users)
            .map_err(|e| BotError::SerializationError(e.to_string()))?;

        fs::write(&self.users_file_path, content)
            .map_err(|e| BotError::AuthError(format!("Failed to save auth file: {}", e)))?;

        Ok(())
    }

    pub fn generate_access_code(&mut self, user_id: Id) -> String {
        let mut rng = rand::rng();
        let random_number = rng.random_range(100000..=999999);
        let code = random_number.to_string();

        self.access_codes.insert(code.clone(), user_id);
        code
    }

    pub fn verify_access_code(
        &mut self,
        code: &str,
        user_id: Id,
    ) -> Result<bool, BotError> {
        if let Some(stored_user_id) = self.access_codes.get(code) {
            if *stored_user_id == user_id {
                self.authorized_users.users.insert(user_id);
                self.access_codes.remove(code);
                self.save_authorized_users()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_authorized(&self, user_id: Id) -> bool {
        self.authorized_users.users.contains(&user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_temp_auth_file() -> NamedTempFile {
        NamedTempFile::new().unwrap()
    }

    #[test]
    fn test_new_creates_manager_with_empty_users_when_file_does_not_exist() {
        let temp_file = create_temp_auth_file();
        let non_existent_path = temp_file.path().with_extension("nonexistent");

        let auth_manager = AuthManager::new(&non_existent_path).unwrap();

        assert!(auth_manager.authorized_users.users.is_empty());
        assert!(auth_manager.access_codes.is_empty());
        assert_eq!(auth_manager.users_file_path, non_existent_path);
    }

    #[test]
    fn test_new_loads_existing_authorized_users() {
        let temp_file = create_temp_auth_file();
        let auth_data = r#"{"users": [123, 456]}"#;
        fs::write(temp_file.path(), auth_data).unwrap();

        let auth_manager = AuthManager::new(temp_file.path()).unwrap();

        assert!(auth_manager.authorized_users.users.contains(&123));
        assert!(auth_manager.authorized_users.users.contains(&456));
        assert_eq!(auth_manager.authorized_users.users.len(), 2);
    }

    #[test]
    fn test_new_handles_invalid_json_gracefully() {
        let temp_file = create_temp_auth_file();
        let invalid_json = r#"{"users": [123, "invalid"]}"#;
        fs::write(temp_file.path(), invalid_json).unwrap();

        let result = AuthManager::new(temp_file.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_access_code_creates_unique_codes() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let code1 = auth_manager.generate_access_code(user_id);
        let code2 = auth_manager.generate_access_code(user_id);

        assert_ne!(code1, code2);
        assert_eq!(code1.len(), 6);
        assert_eq!(code2.len(), 6);
        assert!(code1.chars().all(|c| c.is_ascii_digit()));
        assert!(code2.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_access_code_stores_user_mapping() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let code = auth_manager.generate_access_code(user_id);

        assert_eq!(auth_manager.access_codes.get(&code), Some(&user_id));
        assert_eq!(auth_manager.access_codes.len(), 1);
    }

    #[test]
    fn test_verify_access_code_successful_verification() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let code = auth_manager.generate_access_code(user_id);
        let result = auth_manager.verify_access_code(&code, user_id).unwrap();

        assert!(result);
        assert!(auth_manager.is_authorized(user_id));
        assert!(auth_manager.access_codes.get(&code).is_none());
    }

    #[test]
    fn test_verify_access_code_wrong_user_id() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let correct_user_id = 123;
        let wrong_user_id = 456;

        let code = auth_manager.generate_access_code(correct_user_id);
        let result = auth_manager.verify_access_code(&code, wrong_user_id).unwrap();

        assert!(!result);
        assert!(!auth_manager.is_authorized(correct_user_id));
        assert!(!auth_manager.is_authorized(wrong_user_id));
        assert!(auth_manager.access_codes.get(&code).is_some());
    }

    #[test]
    fn test_verify_access_code_invalid_code() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let result = auth_manager.verify_access_code("000000", user_id).unwrap();

        assert!(!result);
        assert!(!auth_manager.is_authorized(user_id));
    }

    #[test]
    fn test_verify_access_code_removes_code_after_successful_use() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let code = auth_manager.generate_access_code(user_id);
        let initial_code_count = auth_manager.access_codes.len();

        auth_manager.verify_access_code(&code, user_id).unwrap();
        let final_code_count = auth_manager.access_codes.len();

        assert_eq!(initial_code_count, 1);
        assert_eq!(final_code_count, 0);
        assert!(auth_manager.access_codes.get(&code).is_none());
    }

    #[test]
    fn test_verify_access_code_persists_authorization() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user_id = 123;

        let code = auth_manager.generate_access_code(user_id);
        auth_manager.verify_access_code(&code, user_id).unwrap();

        // Create new manager to verify persistence
        let auth_manager2 = AuthManager::new(temp_file.path()).unwrap();
        assert!(auth_manager2.is_authorized(user_id));
    }

    #[test]
    fn test_is_authorized() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let authorized_user = 123;
        let unauthorized_user = 456;

        // Initially no users are authorized
        assert!(!auth_manager.is_authorized(authorized_user));
        assert!(!auth_manager.is_authorized(unauthorized_user));

        // Authorize a user
        let code = auth_manager.generate_access_code(authorized_user);
        auth_manager.verify_access_code(&code, authorized_user).unwrap();

        // Verify authorization status
        assert!(auth_manager.is_authorized(authorized_user));
        assert!(!auth_manager.is_authorized(unauthorized_user));
    }

    #[test]
    fn test_save_authorized_users_creates_file() {
        let temp_file = create_temp_auth_file();
        let path = temp_file.path();

        // Create manager and authorize a user
        let mut auth_manager = AuthManager::new(path).unwrap();
        let user_id = 123;
        let code = auth_manager.generate_access_code(user_id);
        auth_manager.verify_access_code(&code, user_id).unwrap();

        // Verify file was created and contains data
        assert!(path.exists());
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains(&user_id.to_string()));
    }

    #[test]
    fn test_multiple_access_codes_different_users() {
        let temp_file = create_temp_auth_file();
        let mut auth_manager = AuthManager::new(temp_file.path()).unwrap();
        let user1_id = 123;
        let user2_id = 456;

        let code1 = auth_manager.generate_access_code(user1_id);
        let code2 = auth_manager.generate_access_code(user2_id);

        assert_ne!(code1, code2);
        assert_eq!(auth_manager.access_codes.len(), 2);
        assert_eq!(auth_manager.access_codes.get(&code1), Some(&user1_id));
        assert_eq!(auth_manager.access_codes.get(&code2), Some(&user2_id));
    }
}
