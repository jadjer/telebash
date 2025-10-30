use crate::errors::BotError;
use crate::types::{FileItem, Id};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileManager {
    sessions: HashMap<Id, PathBuf>,
}

impl FileManager {
    pub fn new() -> Result<Self, BotError> {
        Ok(FileManager {
            sessions: HashMap::new(),
        })
    }

    pub fn list_directory(&self, user_id: Id) -> Result<Vec<FileItem>, BotError> {
        let current_directory_for_user = self.get_current_directory_for_user(user_id);

        let mut items = Vec::new();

        for entry in fs::read_dir(&current_directory_for_user)
            .map_err(|e| BotError::FileError(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                BotError::FileError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let metadata = entry.metadata().map_err(|e| {
                BotError::FileError(format!("Failed to get metadata for '{}': {}", name, e))
            })?;

            let is_directory = metadata.is_dir();
            let size = metadata.len();

            items.push(FileItem {
                name,
                path,
                is_directory,
                size,
            });
        }

        Ok(items)
    }

    pub fn change_directory(&mut self, user_id: Id, path: &str) -> Result<(), BotError> {
        let current_dir = self.get_current_directory_for_user(user_id);

        let new_path = if path == ".." {
            current_dir
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| current_dir.clone())
        } else {
            current_dir.join(path)
        };

        if new_path.is_dir() {
            let canonical_path = new_path
                .canonicalize()
                .map_err(|e| BotError::FileError(format!("Failed to canonicalize path: {}", e)))?;

            self.sessions.insert(user_id, canonical_path);
            Ok(())
        } else {
            Err(BotError::FileError("Directory does not exist".to_string()))
        }
    }

    pub fn get_current_directory(&self, user_id: Id) -> PathBuf {
        self.get_current_directory_for_user(user_id)
    }

    pub fn get_file_path(&self, user_id: Id, filename: &str) -> PathBuf {
        self.get_current_directory_for_user(user_id).join(filename)
    }

    pub fn file_exists(&self, user_id: Id, filename: &str) -> bool {
        self.get_current_directory_for_user(user_id)
            .join(filename)
            .exists()
    }

    pub fn is_file(&self, user_id: Id, filename: &str) -> bool {
        self.get_current_directory_for_user(user_id)
            .join(filename)
            .is_file()
    }

    pub fn get_current_directory_for_user(&self, user_id: Id) -> PathBuf {
        self.sessions
            .get(&user_id)
            .cloned()
            .unwrap_or_else(|| PathBuf::from("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, FileManager) {
        let temp_dir = TempDir::new().unwrap();
        let file_manager = FileManager::new().unwrap();
        (temp_dir, file_manager)
    }

    fn create_test_files(temp_dir: &TempDir) {
        // Создаем тестовые файлы и директории
        File::create(temp_dir.path().join("file1.txt")).unwrap();
        File::create(temp_dir.path().join("file2.txt")).unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        File::create(temp_dir.path().join("subdir").join("file3.txt")).unwrap();
    }

    #[test]
    fn test_new_file_manager() {
        let file_manager = FileManager::new().unwrap();
        assert!(file_manager.sessions.is_empty());
    }

    #[test]
    fn test_get_current_directory_for_user_default() {
        let file_manager = FileManager::new().unwrap();
        let user_id = 123;

        let path = file_manager.get_current_directory_for_user(user_id);
        assert_eq!(path, PathBuf::from("/"));
    }

    #[test]
    fn test_list_directory() {
        let (temp_dir, file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user_id = 123;

        // Устанавливаем текущую директорию для пользователя
        let mut file_manager = file_manager;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        let items = file_manager.list_directory(user_id).unwrap();

        // Проверяем, что мы получили ожидаемые файлы и директории
        let mut found_file1 = false;
        let mut found_file2 = false;
        let mut found_subdir = false;

        for item in items {
            if item.name == "file1.txt" && !item.is_directory {
                found_file1 = true;
            } else if item.name == "file2.txt" && !item.is_directory {
                found_file2 = true;
            } else if item.name == "subdir" && item.is_directory {
                found_subdir = true;
            }
        }

        assert!(found_file1, "file1.txt should be found");
        assert!(found_file2, "file2.txt should be found");
        assert!(found_subdir, "subdir should be found");
    }

    #[test]
    fn test_list_directory_nonexistent() {
        let mut file_manager = FileManager::new().unwrap();
        let user_id = 123;
        
        // Пытаемся прочитать несуществующую директорию
        file_manager
            .sessions
            .insert(user_id, PathBuf::from("/nonexistent/path"));

        let result = file_manager.list_directory(user_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_change_directory_relative() {
        let (temp_dir, mut file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user_id = 123;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        // Переходим в поддиректорию
        file_manager.change_directory(user_id, "subdir").unwrap();

        let current_dir = file_manager.get_current_directory(user_id);
        assert!(current_dir.ends_with("subdir"));

        // Проверяем, что можем прочитать файлы в новой директории
        let items = file_manager.list_directory(user_id).unwrap();
        let has_file3 = items.iter().any(|item| item.name == "file3.txt");
        assert!(has_file3, "Should find file3.txt in subdir");
    }

    #[test]
    fn test_change_directory_parent() {
        let (temp_dir, mut file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user_id = 123;
        let subdir_path = temp_dir.path().join("subdir");
        file_manager.sessions.insert(user_id, subdir_path.clone());

        // Возвращаемся к родительской директории
        file_manager.change_directory(user_id, "..").unwrap();

        let current_dir = file_manager.get_current_directory(user_id);
        assert_eq!(current_dir, temp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_change_directory_nonexistent() {
        let (temp_dir, mut file_manager) = setup_test_env();
        let user_id = 123;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        let result = file_manager.change_directory(user_id, "nonexistent_dir");
        assert!(result.is_err());
    }

    #[test]
    fn test_change_directory_root_parent() {
        let mut file_manager = FileManager::new().unwrap();
        let user_id = 123;

        // Устанавливаем корневую директорию
        file_manager.sessions.insert(user_id, PathBuf::from("/"));

        // Пытаемся перейти к родительской директории из корня
        file_manager.change_directory(user_id, "..").unwrap();

        // Должны остаться в корне
        let current_dir = file_manager.get_current_directory(user_id);
        assert_eq!(current_dir, PathBuf::from("/"));
    }

    #[test]
    fn test_get_file_path() {
        let (temp_dir, file_manager) = setup_test_env();
        let user_id = 123;

        let mut file_manager = file_manager;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        let file_path = file_manager.get_file_path(user_id, "test_file.txt");
        let expected_path = temp_dir.path().join("test_file.txt");

        assert_eq!(file_path, expected_path);
    }

    #[test]
    fn test_file_exists() {
        let (temp_dir, file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user_id = 123;
        let mut file_manager = file_manager;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        assert!(file_manager.file_exists(user_id, "file1.txt"));
        assert!(!file_manager.file_exists(user_id, "nonexistent.txt"));
    }

    #[test]
    fn test_is_file() {
        let (temp_dir, file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user_id = 123;
        let mut file_manager = file_manager;
        file_manager
            .sessions
            .insert(user_id, temp_dir.path().to_path_buf());

        assert!(file_manager.is_file(user_id, "file1.txt"));
        assert!(!file_manager.is_file(user_id, "subdir")); // subdir - это директория
    }

    #[test]
    fn test_multiple_users() {
        let (temp_dir, mut file_manager) = setup_test_env();
        create_test_files(&temp_dir);

        let user1_id = 123;
        let user2_id = 456;

        // Устанавливаем разные директории для разных пользователей
        file_manager
            .sessions
            .insert(user1_id, temp_dir.path().to_path_buf());
        file_manager
            .sessions
            .insert(user2_id, temp_dir.path().join("subdir"));

        // Проверяем, что пользователи имеют разные текущие директории
        let user1_dir = file_manager.get_current_directory(user1_id);
        let user2_dir = file_manager.get_current_directory(user2_id);

        assert_ne!(user1_dir, user2_dir);
        assert!(user2_dir.ends_with("subdir"));

        // Проверяем, что изменение директории одного пользователя не влияет на другого
        file_manager.change_directory(user1_id, "subdir").unwrap();

        let user1_dir_after = file_manager.get_current_directory(user1_id);
        let user2_dir_after = file_manager.get_current_directory(user2_id);

        assert_eq!(user1_dir_after, user2_dir_after); // Теперь оба в subdir

        // Меняем директорию только для user2
        file_manager.change_directory(user2_id, "..").unwrap();

        let user1_dir_final = file_manager.get_current_directory(user1_id);
        let user2_dir_final = file_manager.get_current_directory(user2_id);

        assert_ne!(user1_dir_final, user2_dir_final);
        assert!(user1_dir_final.ends_with("subdir"));
        assert_eq!(user2_dir_final, temp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_file_item_properties() {
        let (temp_dir, _) = setup_test_env();
        create_test_files(&temp_dir);

        // Проверяем свойства FileItem напрямую
        let entries: Vec<_> = fs::read_dir(temp_dir.path()).unwrap().collect();
        let entry = entries[0].as_ref().unwrap();

        let file_item = FileItem {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            is_directory: entry.metadata().unwrap().is_dir(),
            size: entry.metadata().unwrap().len(),
        };

        assert!(!file_item.name.is_empty());
        assert!(file_item.path.exists());
        // is_directory и size проверяются косвенно через другие тесты
    }
}
