use edge_workers::RetryPolicy;
use proptest::prelude::*;
use std::time::Duration;

proptest! {
    #[test]
    fn prop_backoff_delay_never_exceeds_max(
        max_retries in 1u32..20,
        initial_ms in 1u64..1000,
        max_ms in 1000u64..100_000,
        multiplier in 2u32..5,
        attempt in 0u32..50
    ) {
        let policy = RetryPolicy::new(
            max_retries,
            Duration::from_millis(initial_ms),
            Duration::from_millis(max_ms),
            multiplier,
        );

        let delay = policy.calculate_delay(attempt);
        prop_assert!(delay <= Duration::from_millis(max_ms));

        if attempt == 0 {
            prop_assert_eq!(delay, Duration::ZERO);
        } else if attempt == 1 {
            prop_assert_eq!(delay, Duration::from_millis(initial_ms).min(Duration::from_millis(max_ms)));
        }
    }

    #[test]
    fn prop_backoff_delay_monotonic(
        max_retries in 1u32..10,
        initial_ms in 1u64..50,
        max_ms in 500u64..10_000,
        multiplier in 2u32..4,
        attempt in 1u32..10
    ) {
        let policy = RetryPolicy::new(
            max_retries,
            Duration::from_millis(initial_ms),
            Duration::from_millis(max_ms),
            multiplier,
        );

        let delay1 = policy.calculate_delay(attempt);
        let delay2 = policy.calculate_delay(attempt + 1);
        prop_assert!(delay1 <= delay2);
    }
}
