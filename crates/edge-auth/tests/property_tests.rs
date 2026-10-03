use edge_auth::{Claims, JwtKeyPair, TokenType};
use proptest::prelude::*;
use uuid::Uuid;

proptest! {
    #[test]
    fn prop_jwt_roundtrip(
        ttl in 100i64..10000i64,
        role in "[a-z]{3,10}",
    ) {
        let keypair = JwtKeyPair::generate();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access(user_id, vec![role.clone()], ttl);

        let token = keypair.sign(&claims).expect("signing must succeed");
        let verified = keypair.verify(&token).expect("verification must succeed");

        prop_assert_eq!(verified.sub, user_id);
        prop_assert_eq!(verified.roles, vec![role]);
        prop_assert_eq!(verified.token_type, TokenType::Access);
    }
}
