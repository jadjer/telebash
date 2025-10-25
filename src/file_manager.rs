use crate::errors::BotError;
use crate::types::FileItem;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileManager {
    current_directory: PathBuf,
}

impl FileManager {
    pub fn new(working_directory: &str) -> Result<Self, BotError> {
        let path = PathBuf::from(working_directory);

        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| BotError::FileError(format!("Failed to create working directory: {}", e)))?;
        }

        Ok(FileManager {
            current_directory: path.canonicalize()
                .map_err(|e| BotError::FileError(format!("Failed to canonicalize path: {}", e)))?,
        })
    }

    pub fn list_directory(&self) -> Result<Vec<FileItem>, BotError> {
        let mut items = Vec::new();

        for entry in fs::read_dir(&self.current_directory)
            .map_err(|e| BotError::FileError(format!("Failed to read directory: {}", e)))? {

            let entry = entry
                .map_err(|e| BotError::FileError(format!("Failed to read directory entry: {}", e)))?;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let is_directory = path.is_dir();

            items.push(FileItem {
                name,
                path,
                is_directory,
            });
        }

        Ok(items)
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), BotError> {
        let new_path = if path == ".." {
            self.current_directory.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.current_directory.clone())
        } else {
            self.current_directory.join(path)
        };

        if new_path.is_dir() {
            self.current_directory = new_path.canonicalize()
                .map_err(|e| BotError::FileError(format!("Failed to canonicalize path: {}", e)))?;
            Ok(())
        } else {
            Err(BotError::FileError("Directory does not exist".to_string()))
        }
    }

    pub fn get_current_directory(&self) -> &Path {
        &self.current_directory
    }

    pub fn get_file_path(&self, filename: &str) -> PathBuf {
        self.current_directory.join(filename)
    }

    pub fn file_exists(&self, filename: &str) -> bool {
        self.current_directory.join(filename).exists()
    }

    pub fn is_file(&self, filename: &str) -> bool {
        self.current_directory.join(filename).is_file()
    }
}