//! EdgeArena authentication, authorization, and cryptographic token services.
//!
//! Provides asymmetric Ed25519 JWT signing/verification, Argon2id password hashing,
//! and Role-Based Access Control (RBAC) permission evaluation.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod jwt;
pub mod password;
pub mod rbac;

pub use error::AuthError;
pub use jwt::{Claims, JwtKeyPair, TokenType};
pub use password::{hash_password, verify_password};
pub use rbac::{permissions, RbacAuthorizer};

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use uuid::Uuid;

    #[test]
    fn test_password_hash_and_verify() {
        let raw_password = "SecureSuperSecret123!";
        let hash = hash_password(raw_password).expect("hashing must succeed");
        assert!(hash.starts_with("$argon2id$"));

        let ok = verify_password(raw_password, &hash).expect("verification should run");
        assert!(ok, "matching password must verify");

        let bad = verify_password("WrongPassword!", &hash).expect("verification should run");
        assert!(!bad, "wrong password must fail");
    }

    #[test]
    fn test_jwt_sign_and_verify() {
        let keypair = JwtKeyPair::generate();
        let user_id = Uuid::new_v4();
        let roles = vec!["analyst".to_string(), "auditor".to_string()];
        let claims = Claims::new_access(user_id, roles.clone(), 3600);

        let token = keypair.sign(&claims).expect("signing must succeed");
        assert_eq!(token.matches('.').count(), 2);

        let verified = keypair.verify(&token).expect("verification must succeed");
        assert_eq!(verified.sub, user_id);
        assert_eq!(verified.roles, roles);
        assert_eq!(verified.token_type, TokenType::Access);
    }

    #[test]
    fn test_jwt_expired_token_rejected() {
        let keypair = JwtKeyPair::generate();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access(user_id, vec!["analyst".into()], -10);

        let token = keypair.sign(&claims).expect("sign expired");
        let err = keypair.verify(&token).unwrap_err();
        assert_eq!(err, AuthError::ExpiredToken);
    }

    #[test]
    fn test_jwt_tampered_payload_rejected() {
        let keypair = JwtKeyPair::generate();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access(user_id, vec!["trader".into()], 3600);

        let token = keypair.sign(&claims).expect("sign token");
        let parts: Vec<&str> = token.split('.').collect();
        // Tamper signature by flipping a bit in decoded signature bytes
        let mut sig_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[2])
            .unwrap();
        sig_bytes[0] ^= 0x01;
        let tampered_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&sig_bytes);

        let tampered_token = format!("{}.{}.{}", parts[0], parts[1], tampered_sig);
        let err = keypair.verify(&tampered_token).unwrap_err();
        assert_eq!(err, AuthError::InvalidSignature);
    }

    #[test]
    fn test_jwt_wrong_key_rejected() {
        let keypair_a = JwtKeyPair::generate();
        let keypair_b = JwtKeyPair::generate();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access(user_id, vec!["admin".into()], 3600);

        let token = keypair_a.sign(&claims).unwrap();
        let err = keypair_b.verify(&token).unwrap_err();
        assert_eq!(err, AuthError::InvalidSignature);
    }

    #[test]
    fn test_rbac_authorization() {
        let rbac = RbacAuthorizer::default();

        // Admin can access everything
        assert!(rbac.authorize(&["admin"], permissions::RULES_WRITE).is_ok());
        assert!(rbac.authorize(&["admin"], "any:arbitrary:perm").is_ok());

        // Analyst can write rules and read events
        assert!(rbac
            .authorize(&["analyst"], permissions::RULES_WRITE)
            .is_ok());
        assert!(rbac
            .authorize(&["analyst"], permissions::EVENTS_READ)
            .is_ok());
        // Analyst cannot manage users
        assert!(rbac
            .authorize(&["analyst"], permissions::USERS_MANAGE)
            .is_err());

        // Trader can write events but not write rules
        assert!(rbac
            .authorize(&["trader"], permissions::EVENTS_WRITE)
            .is_ok());
        assert!(rbac
            .authorize(&["trader"], permissions::RULES_WRITE)
            .is_err());

        // Unknown role rejected
        assert!(rbac
            .authorize(&["guest"], permissions::EVENTS_READ)
            .is_err());
    }
}
