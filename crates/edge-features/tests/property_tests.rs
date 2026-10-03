use edge_domain::Channel;
use edge_features::EventWindow;
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn prop_event_window_sum_monotonic_with_positive_amounts(
        amounts in proptest::collection::vec(1i64..100_000i64, 1..50),
    ) {
        let mut window = EventWindow::new();
        let mut expected_sum = 0i64;

        for (i, &amt) in amounts.iter().enumerate() {
            window.record(i as i64, amt, Channel::Web, BTreeMap::new());
            expected_sum += amt;
        }

        prop_assert_eq!(window.count_since(0), amounts.len() as i64);
        prop_assert_eq!(window.sum_since(0), expected_sum);
    }
}
