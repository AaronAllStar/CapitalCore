//! Declarative OpenAPI 3.0 specification generation for EdgeArena API.

use serde_json::json;

/// Generates the canonical OpenAPI 3.0 specification document for the EdgeArena API.
#[must_use]
pub fn generate_openapi_spec() -> serde_json::Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "EdgeArena Financial Intelligence Engine API",
            "version": "1.0.0",
            "description": "High-throughput fraud detection, real-time transaction monitoring, and auditable policy enforcement."
        },
        "paths": {
            "/health/liveness": {
                "get": {
                    "summary": "Kubernetes liveness probe",
                    "responses": {
                        "200": { "description": "Process is alive and responding" }
                    }
                }
            },
            "/health/readiness": {
                "get": {
                    "summary": "Subsystem readiness evaluation",
                    "responses": {
                        "200": { "description": "All subsystems healthy" },
                        "503": { "description": "One or more critical subsystems unhealthy" }
                    }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "Point-in-time runtime metrics snapshot",
                    "responses": {
                        "200": { "description": "Structured JSON metric counters and gauges" }
                    }
                }
            },
            "/api/v1/transactions": {
                "post": {
                    "summary": "Submit a financial transaction for real-time risk assessment",
                    "security": [{ "BearerAuth": [] }],
                    "responses": {
                        "200": { "description": "Transaction assessed and decision emitted" },
                        "400": { "description": "Invalid payload format" },
                        "401": { "description": "Missing or invalid authorization token" },
                        "409": { "description": "Duplicate event submission detected" }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        }
    })
}
