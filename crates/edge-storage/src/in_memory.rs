//! In-memory thread-safe user repository implementation for tests and local development.

use crate::error::StorageError;
use crate::models::UserRecord;
use crate::repository::{BoxFuture, UserRepository};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Thread-safe in-memory implementation of `UserRepository` for testing and offline environments.
#[derive(Default)]
pub struct InMemoryUserRepository {
    records: RwLock<HashMap<Uuid, UserRecord>>,
}

impl InMemoryUserRepository {
    /// Creates a new empty `InMemoryUserRepository`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn create<'a>(
        &'a self,
        mut user: UserRecord,
    ) -> BoxFuture<'a, Result<UserRecord, StorageError>> {
        Box::pin(async move {
            let mut records = self
                .records
                .write()
                .map_err(|e| StorageError::Database(format!("lock error: {e}")))?;

            if records.contains_key(&user.id) {
                return Err(StorageError::Duplicate(format!(
                    "user with id {} already exists",
                    user.id
                )));
            }

            for existing in records.values() {
                if existing.email.eq_ignore_ascii_case(&user.email) {
                    return Err(StorageError::Duplicate(format!(
                        "user with email {} already exists",
                        user.email
                    )));
                }
            }

            let now = Utc::now();
            user.created_at = now;
            user.updated_at = now;

            records.insert(user.id, user.clone());
            Ok(user)
        })
    }

    fn find_by_id<'a>(
        &'a self,
        id: Uuid,
    ) -> BoxFuture<'a, Result<Option<UserRecord>, StorageError>> {
        Box::pin(async move {
            let records = self
                .records
                .read()
                .map_err(|e| StorageError::Database(format!("lock error: {e}")))?;

            Ok(records.get(&id).cloned())
        })
    }

    fn find_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> BoxFuture<'a, Result<Option<UserRecord>, StorageError>> {
        Box::pin(async move {
            let records = self
                .records
                .read()
                .map_err(|e| StorageError::Database(format!("lock error: {e}")))?;

            let found = records
                .values()
                .find(|u| u.email.eq_ignore_ascii_case(email))
                .cloned();

            Ok(found)
        })
    }

    fn update<'a>(
        &'a self,
        mut user: UserRecord,
    ) -> BoxFuture<'a, Result<UserRecord, StorageError>> {
        Box::pin(async move {
            let mut records = self
                .records
                .write()
                .map_err(|e| StorageError::Database(format!("lock error: {e}")))?;

            if !records.contains_key(&user.id) {
                return Err(StorageError::NotFound(format!(
                    "user with id {} not found",
                    user.id
                )));
            }

            // Check email uniqueness on update
            for existing in records.values() {
                if existing.id != user.id && existing.email.eq_ignore_ascii_case(&user.email) {
                    return Err(StorageError::Duplicate(format!(
                        "email {} is already taken by another account",
                        user.email
                    )));
                }
            }

            user.updated_at = Utc::now();
            records.insert(user.id, user.clone());
            Ok(user)
        })
    }

    fn delete<'a>(&'a self, id: Uuid) -> BoxFuture<'a, Result<bool, StorageError>> {
        Box::pin(async move {
            let mut records = self
                .records
                .write()
                .map_err(|e| StorageError::Database(format!("lock error: {e}")))?;

            Ok(records.remove(&id).is_some())
        })
    }
}
