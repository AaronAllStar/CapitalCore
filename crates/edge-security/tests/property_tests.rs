use edge_security::validate_length;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_validate_length_consistent(
        s in "[a-zA-Z0-9]{0,50}",
        min in 0usize..25usize,
        max in 25usize..50usize,
    ) {
        let count = s.chars().count();
        let res = validate_length("prop_field", &s, min, max);

        if count >= min && count <= max {
            prop_assert!(res.is_ok());
        } else {
            prop_assert!(res.is_err());
        }
    }
}
