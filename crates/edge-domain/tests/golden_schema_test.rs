use edge_domain::{Decision, EventId, FinancialEvent, RiskLevel, RuleId};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ExpectedDecision {
    event_id: EventId,
    decision: Decision,
    risk_level: RiskLevel,
    reasons: Vec<String>,
    triggered_rules: Vec<RuleId>,
    model_id: Option<String>,
}

#[test]
fn test_golden_dataset_parses_valid_financial_events() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // root
    path.push("golden");
    path.push("datasets");
    path.push("001_baseline_events.jsonl");

    let file = File::open(&path).expect("open golden dataset file");
    let reader = BufReader::new(file);

    let mut event_count = 0;
    for line in reader.lines() {
        let line = line.expect("read line");
        if line.trim().is_empty() {
            continue;
        }
        let event: FinancialEvent = serde_json::from_str(&line).unwrap_or_else(|e| {
            panic!("Failed to parse line as FinancialEvent: {line} - Error: {e}")
        });
        assert!(!event.event_id().as_uuid().is_nil());
        event_count += 1;
    }

    assert_eq!(event_count, 10, "Expected 10 golden test events in dataset");
}

#[test]
fn test_golden_expected_parses_all_decision_types() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // root
    path.push("golden");
    path.push("expected");
    path.push("001_baseline_decisions.jsonl");

    let file = File::open(&path).expect("open golden expected file");
    let reader = BufReader::new(file);

    let mut has_allow = false;
    let mut has_review = false;
    let mut has_block = false;
    let mut has_escalate = false;
    let mut total_count = 0;

    for line in reader.lines() {
        let line = line.expect("read line");
        if line.trim().is_empty() {
            continue;
        }
        let expected: ExpectedDecision = serde_json::from_str(&line).unwrap_or_else(|e| {
            panic!("Failed to parse line as ExpectedDecision: {line} - Error: {e}")
        });

        match expected.decision {
            Decision::Allow => has_allow = true,
            Decision::Review => has_review = true,
            Decision::Block => has_block = true,
            Decision::Escalate => has_escalate = true,
        }
        total_count += 1;
    }

    assert_eq!(total_count, 10);
    assert!(has_allow, "Golden dataset must include ALLOW outcome");
    assert!(has_review, "Golden dataset must include REVIEW outcome");
    assert!(has_block, "Golden dataset must include BLOCK outcome");
    assert!(has_escalate, "Golden dataset must include ESCALATE outcome");
}
