//! Automated secret and sensitive data scrubbing for logs, audits, and error payloads.

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

static TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static PAN_REGEX: OnceLock<Regex> = OnceLock::new();
static JSON_SECRET_REGEX: OnceLock<Regex> = OnceLock::new();

fn token_regex() -> &'static Regex {
    TOKEN_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(bearer\s+)[A-Za-z0-9\-_\.]{16,}").expect("valid token regex")
    })
}

fn pan_regex() -> &'static Regex {
    PAN_REGEX.get_or_init(|| Regex::new(r"\b(?:\d{4}[ -]?){3}\d{4}\b").expect("valid pan regex"))
}

fn json_secret_regex() -> &'static Regex {
    JSON_SECRET_REGEX.get_or_init(|| {
        Regex::new(
            r#"(?i)("?(?:password|secret|api_key|token|access_token|private_key)"?\s*:\s*)"[^"]+""#,
        )
        .expect("valid json secret regex")
    })
}

/// Scrub string of all detected secrets, JWT tokens, credit card PANs, and sensitive fields.
#[must_use]
pub fn scrub_string(input: &str) -> String {
    let s1 = token_regex().replace_all(input, "${1}[REDACTED_TOKEN]");
    let s2 = pan_regex().replace_all(&s1, "[REDACTED_PAN]");
    let s3 = json_secret_regex().replace_all(&s2, r#"${1}"[REDACTED]""#);
    s3.into_owned()
}

/// Recursively scrubs sensitive keys in a JSON Value tree.
pub fn scrub_json_value(val: &mut Value) {
    match val {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_lowercase();
                if lower.contains("password")
                    || lower.contains("secret")
                    || lower.contains("token")
                    || lower.contains("api_key")
                    || lower.contains("authorization")
                {
                    *v = Value::String("[REDACTED]".to_string());
                } else {
                    scrub_json_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                scrub_json_value(item);
            }
        }
        Value::String(s) => {
            *s = scrub_string(s);
        }
        _ => {}
    }
}
