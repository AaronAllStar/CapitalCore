use edge_security::{
    sanitize_control_chars, scrub_string, validate_email, validate_length, validate_not_empty,
};
use proptest::prelude::*;

#[test]
fn test_fuzz_known_malicious_payloads() {
    let payloads = [
        // SQL injection vectors
        "' OR '1'='1",
        "1; DROP TABLE users; --",
        "admin' --",
        "' UNION SELECT * FROM users --",
        // XSS vectors
        "<script>alert('xss')</script>",
        "<img src=x onerror=alert(1)>",
        "\"><script>document.cookie</script>",
        // Path traversal
        "../../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\cmd.exe",
        // Format string attacks
        "%s%s%s%s%n%x%d",
        "%p%p%p%p",
        // Null bytes and CRLF
        "admin\x00extra",
        "header_val\r\nSet-Cookie: malicious=true",
        "\x01\x02\x03\x04\x05\x06\x07\x08",
    ];

    for payload in payloads {
        // Sanitize must never panic
        let clean = sanitize_control_chars(payload);
        assert!(!clean.contains('\0'));
        assert!(!clean.contains('\x07'));

        // Scrubber must never panic
        let scrubbed = scrub_string(payload);
        assert!(!scrubbed.is_empty());

        // Validate not empty should handle malicious strings safely
        let _ = validate_not_empty("fuzz_field", payload);

        // Validate length should never panic
        let _ = validate_length("fuzz_field", payload, 1, 100);

        // Validate email should reject obvious injections
        assert!(validate_email("email", payload).is_err());
    }
}

proptest! {
    #[test]
    fn prop_fuzz_sanitize_control_chars(input in "\\PC*") {
        let clean = sanitize_control_chars(&input);
        for c in clean.chars() {
            prop_assert!(!c.is_control() || c == '\n' || c == '\r' || c == '\t');
        }
    }

    #[test]
    fn prop_fuzz_scrub_string_never_panics(input in "\\PC*") {
        let scrubbed = scrub_string(&input);
        // Scrubbed output length is always non-negative
        prop_assert!(scrubbed.len() <= input.len().max(1) * 3);
    }

    #[test]
    fn prop_fuzz_validate_length_bounds(
        input in "\\PC*",
        min in 0usize..50,
        max in 50usize..200
    ) {
        let res = validate_length("field", &input, min, max);
        let char_len = input.chars().count();
        if char_len >= min && char_len <= max {
            prop_assert!(res.is_ok());
        } else {
            prop_assert!(res.is_err());
        }
    }
}
