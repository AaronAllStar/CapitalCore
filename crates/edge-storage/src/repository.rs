//! Repository interfaces and traits for persistence.

use crate::error::StorageError;
use crate::models::UserRecord;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

/// Type alias for thread-safe asynchronous pinned futures.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Repository interface for user account persistence.
pub trait UserRepository: Send + Sync {
    /// Inserts a new user record. Returns error on duplicate email or ID.
    fn create<'a>(&'a self, user: UserRecord) -> BoxFuture<'a, Result<UserRecord, StorageError>>;

    /// Finds a user record by its primary key UUID.
    fn find_by_id<'a>(
        &'a self,
        id: Uuid,
    ) -> BoxFuture<'a, Result<Option<UserRecord>, StorageError>>;

    /// Finds a user record by case-insensitive email address.
    fn find_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> BoxFuture<'a, Result<Option<UserRecord>, StorageError>>;

    /// Updates an existing user record. Returns `StorageError::NotFound` if user does not exist.
    fn update<'a>(&'a self, user: UserRecord) -> BoxFuture<'a, Result<UserRecord, StorageError>>;

    /// Deletes a user by ID. Returns `true` if record was deleted.
    fn delete<'a>(&'a self, id: Uuid) -> BoxFuture<'a, Result<bool, StorageError>>;
}
