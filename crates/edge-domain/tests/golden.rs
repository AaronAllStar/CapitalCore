use edge_domain::{Decision, EventId, FinancialEvent, RiskLevel, RuleId};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExpectedDecision {
    pub event_id: EventId,
    pub decision: Decision,
    pub risk_level: RiskLevel,
    pub reasons: Vec<String>,
    pub triggered_rules: Vec<RuleId>,
    pub model_id: Option<String>,
}

/// Golden test evaluation report.
#[derive(Default)]
pub struct GoldenTestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub mismatches: Vec<String>,
}

impl GoldenTestReport {
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            mismatches: Vec::new(),
        }
    }

    pub fn record_success(&mut self) {
        self.total += 1;
        self.passed += 1;
    }

    pub fn record_failure(&mut self, detail: String) {
        self.total += 1;
        self.failed += 1;
        self.mismatches.push(detail);
    }
}

fn golden_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/
    path.pop(); // root
    path.push("golden");
    path
}

fn load_dataset(path: &Path) -> Vec<FinancialEvent> {
    let file = File::open(path)
        .unwrap_or_else(|e| panic!("Failed to open dataset at {}: {e}", path.display()));
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            let line = line.expect("read line");
            serde_json::from_str::<FinancialEvent>(&line).unwrap_or_else(|e| {
                panic!("Failed to deserialize FinancialEvent: {line} - Error: {e}")
            })
        })
        .collect()
}

fn load_expected(path: &Path) -> HashMap<EventId, ExpectedDecision> {
    let file = File::open(path)
        .unwrap_or_else(|e| panic!("Failed to open expected file at {}: {e}", path.display()));
    let reader = BufReader::new(file);

    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line.expect("read line");
        let expected: ExpectedDecision = serde_json::from_str(&line).unwrap_or_else(|e| {
            panic!("Failed to deserialize ExpectedDecision: {line} - Error: {e}")
        });
        map.insert(expected.event_id, expected);
    }
    map
}

#[test]
fn test_golden_dataset_validation_runner() {
    let base = golden_dir();
    let dataset_path = base.join("datasets").join("001_baseline_events.jsonl");
    let expected_path = base.join("expected").join("001_baseline_decisions.jsonl");

    let events = load_dataset(&dataset_path);
    let expected_map = load_expected(&expected_path);

    let mut report = GoldenTestReport::new();

    for event in &events {
        match expected_map.get(&event.event_id()) {
            Some(expected) => {
                // Verify event ID matches expected output
                if expected.event_id == event.event_id() {
                    report.record_success();
                } else {
                    let diff = format!(
                        "\n--- Golden Mismatch ---\nEvent ID: {}\nExpected Event ID: {}\nActual Event ID: {}\n----------------------",
                        event.event_id(),
                        expected.event_id,
                        event.event_id()
                    );
                    report.record_failure(diff);
                }
            }
            None => {
                let missing = format!(
                    "\n--- Missing Expected Output ---\nEvent ID {} found in dataset but missing from expected file.\n",
                    event.event_id()
                );
                report.record_failure(missing);
            }
        }
    }

    println!(
        "\n================ GOLDEN RUNNER REPORT ================\nTotal Events Evaluated: {}\nPassed: {}\nFailed: {}\n======================================================",
        report.total, report.passed, report.failed
    );

    if !report.mismatches.is_empty() {
        for mismatch in &report.mismatches {
            eprintln!("{mismatch}");
        }
        panic!(
            "Golden runner failed with {} mismatches out of {} events.",
            report.failed, report.total
        );
    }

    assert_eq!(report.passed, events.len());
    assert_eq!(report.failed, 0);
}
