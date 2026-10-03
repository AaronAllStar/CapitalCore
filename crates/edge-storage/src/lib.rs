//! EdgeArena persistence and relational storage abstraction.
//!
//! Provides traits and records for users, RBAC, and audit events,
//! along with an in-memory test double and PostgreSQL connection pool configuration.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod in_memory;
pub mod models;
pub mod pool;
pub mod repository;

pub use error::StorageError;
pub use in_memory::InMemoryUserRepository;
pub use models::{PermissionRecord, RoleRecord, UserRecord};
pub use pool::{run_migrations, PgPoolConfig};
pub use repository::{BoxFuture, UserRepository};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_and_find_user() {
        let repo = InMemoryUserRepository::new();
        let user_id = Uuid::new_v4();
        let user = UserRecord::new(
            user_id,
            "analyst@edgearena.io".to_string(),
            "hashed_argon2_val".to_string(),
            true,
        );

        let created = repo.create(user).await.expect("creation should succeed");
        assert_eq!(created.id, user_id);
        assert_eq!(created.email, "analyst@edgearena.io");

        let found_by_id = repo
            .find_by_id(user_id)
            .await
            .expect("lookup should succeed")
            .expect("user should exist");
        assert_eq!(found_by_id.email, "analyst@edgearena.io");

        let found_by_email = repo
            .find_by_email("ANALYST@edgearena.io")
            .await
            .expect("lookup should succeed")
            .expect("user should exist (case insensitive)");
        assert_eq!(found_by_email.id, user_id);
    }

    #[tokio::test]
    async fn test_duplicate_user_rejected() {
        let repo = InMemoryUserRepository::new();
        let id1 = Uuid::new_v4();
        let user1 = UserRecord::new(
            id1,
            "trader@edgearena.io".to_string(),
            "hash1".to_string(),
            true,
        );
        repo.create(user1).await.expect("create user1");

        let id2 = Uuid::new_v4();
        let user2 = UserRecord::new(
            id2,
            "TRADER@edgearena.io".to_string(),
            "hash2".to_string(),
            true,
        );
        let err = repo.create(user2).await.unwrap_err();
        match err {
            StorageError::Duplicate(_) => {}
            other => panic!("expected StorageError::Duplicate, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_update_and_delete_user() {
        let repo = InMemoryUserRepository::new();
        let user_id = Uuid::new_v4();
        let user = UserRecord::new(
            user_id,
            "operator@edgearena.io".to_string(),
            "hash".to_string(),
            true,
        );
        repo.create(user).await.expect("create operator");

        let mut fetched = repo.find_by_id(user_id).await.unwrap().expect("must exist");
        fetched.email = "lead_operator@edgearena.io".to_string();
        fetched.is_active = false;

        let updated = repo.update(fetched).await.expect("update operator");
        assert_eq!(updated.email, "lead_operator@edgearena.io");
        assert!(!updated.is_active);

        let deleted = repo.delete(user_id).await.expect("delete operator");
        assert!(deleted);

        let missing = repo.find_by_id(user_id).await.unwrap();
        assert!(missing.is_none());

        let delete_again = repo.delete(user_id).await.expect("delete non-existent");
        assert!(!delete_again);
    }
}
