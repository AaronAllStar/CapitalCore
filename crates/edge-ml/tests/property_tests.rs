use edge_ml::FraudModel;
use proptest::prelude::*;
use std::collections::BTreeMap;

const MANIFEST_JSON: &str = include_str!("../../../ml/models/model_manifest.json");

proptest! {
    #[test]
    fn prop_fraud_score_and_bps_bounded(
        amount in 100i64..5_000_000,
        txn_5m in 0i64..100,
        sum_5m in 0i64..10_000_000,
        txn_1h in 0i64..500,
        sum_1h in 0i64..50_000_000,
        max_1h in 0i64..10_000_000,
        txn_24h in 0i64..2000,
        sum_24h in 0i64..200_000_000,
        merchants in 0i64..50
    ) {
        let model = FraudModel::load_from_json(MANIFEST_JSON).unwrap();
        let mut features = BTreeMap::new();
        features.insert("amount_minor".to_string(), amount);
        features.insert("user_txn_count_5m".to_string(), txn_5m);
        features.insert("user_txn_sum_5m".to_string(), sum_5m);
        features.insert("user_txn_count_1h".to_string(), txn_1h);
        features.insert("user_txn_sum_1h".to_string(), sum_1h);
        features.insert("user_txn_max_1h".to_string(), max_1h);
        features.insert("user_txn_count_24h".to_string(), txn_24h);
        features.insert("user_txn_sum_24h".to_string(), sum_24h);
        features.insert("user_distinct_merchants_24h".to_string(), merchants);

        let score = model.predict_score(&features).unwrap();
        prop_assert!((0.0..=1.0).contains(&score));

        let bps = model.predict_risk_bps(&features).unwrap();
        prop_assert!(bps <= 10_000);
        prop_assert_eq!(bps, (score * 10_000.0).round() as u32);
    }
}
