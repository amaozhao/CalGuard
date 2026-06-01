use crate::error::AppError;
use calguard_store::{Database, Repository};
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    repository: Mutex<Repository>,
}

impl AppState {
    pub fn open(database_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let database = Database::open(database_path).map_err(AppError::db)?;
        let repository = Repository::new(database).map_err(AppError::db)?;
        Ok(Self {
            repository: Mutex::new(repository),
        })
    }

    pub fn with_repository<T>(
        &self,
        f: impl FnOnce(&Repository) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let repository = self.repository.lock().map_err(|_| {
            AppError::new(
                "DB_ERROR",
                "本地数据库访问失败。",
                Some("repository lock poisoned".to_string()),
            )
        })?;
        f(&repository)
    }
}
