//! Standard HTTP security response headers protecting against clickjacking, MIME sniffing, and XSS.

/// Canonical security header names and strict production values.
pub mod header_values {
    /// HTTP Strict Transport Security (HSTS) with 2-year duration and preloading.
    pub const STRICT_TRANSPORT_SECURITY: &str = "max-age=63072000; includeSubDomains; preload";

    /// Prevents browsers from MIME-sniffing away from declared Content-Type.
    pub const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";

    /// Completely denies framing to protect against clickjacking attacks.
    pub const X_FRAME_OPTIONS: &str = "DENY";

    /// Strict Content Security Policy forbidding arbitrary external script execution.
    pub const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; frame-ancestors 'none'";

    /// Legacy XSS filter activation in blocking mode.
    pub const X_XSS_PROTECTION: &str = "1; mode=block";

    /// Protects referrer leakage to third-party domains.
    pub const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";

    /// Restricts dangerous browser features and device APIs.
    pub const PERMISSIONS_POLICY: &str =
        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), microphone=(), payment=()";
}

/// Returns a collection of all standard HTTP security headers as `(&'static str, &'static str)` pairs.
#[must_use]
pub fn standard_security_headers() -> [(&'static str, &'static str); 7] {
    [
        (
            "strict-transport-security",
            header_values::STRICT_TRANSPORT_SECURITY,
        ),
        (
            "x-content-type-options",
            header_values::X_CONTENT_TYPE_OPTIONS,
        ),
        ("x-frame-options", header_values::X_FRAME_OPTIONS),
        (
            "content-security-policy",
            header_values::CONTENT_SECURITY_POLICY,
        ),
        ("x-xss-protection", header_values::X_XSS_PROTECTION),
        ("referrer-policy", header_values::REFERRER_POLICY),
        ("permissions-policy", header_values::PERMISSIONS_POLICY),
    ]
}
