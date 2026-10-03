use edge_transactions::{normalize_transaction, RawTransaction};
use proptest::prelude::*;
use uuid::Uuid;

proptest! {
    #[test]
    fn prop_normalize_positive_amounts(
        amount in 1i64..1_000_000_000i64,
    ) {
        let raw = RawTransaction {
            event_id: None,
            transaction_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "payment".to_string(),
            channel: "web".to_string(),
            amount_minor: amount,
            currency: "USD".to_string(),
            timestamp: None,
            metadata: None,
        };

        let event = normalize_transaction(raw).expect("positive amount must normalize");
        prop_assert_eq!(event.money().minor_units(), amount);
    }
}
