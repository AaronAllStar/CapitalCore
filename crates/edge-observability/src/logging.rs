//! Tracing subscriber initialization and configuration for structured logging.

use thiserror::Error;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

/// Tracing and logging initialization errors.
#[derive(Debug, Error)]
pub enum TracingInitError {
    /// Failed to set global default subscriber.
    #[error("failed to set global default tracing subscriber: {0}")]
    SetGlobalDefault(String),
}

/// Output formatting mode for tracing spans and events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Machine-readable structured JSON format.
    #[default]
    Json,
    /// Human-readable colored console format.
    Pretty,
}

/// Observability and logging configuration options.
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Filter directives string (e.g. "info,edge_arena=debug").
    pub env_filter: String,
    /// Serialization format for event logs.
    pub format: LogFormat,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            env_filter: "info".to_string(),
            format: LogFormat::Json,
        }
    }
}

impl ObservabilityConfig {
    /// Creates a configuration with the given level and format.
    #[must_use]
    pub fn new(env_filter: impl Into<String>, format: LogFormat) -> Self {
        Self {
            env_filter: env_filter.into(),
            format,
        }
    }

    /// Initializes the global tracing subscriber according to this configuration.
    pub fn init_subscriber(&self) -> Result<(), TracingInitError> {
        let filter =
            EnvFilter::try_new(&self.env_filter).unwrap_or_else(|_| EnvFilter::new("info"));

        match self.format {
            LogFormat::Json => {
                let subscriber = tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().json());
                subscriber
                    .try_init()
                    .map_err(|e| TracingInitError::SetGlobalDefault(e.to_string()))?;
            }
            LogFormat::Pretty => {
                let subscriber = tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt::layer().pretty());
                subscriber
                    .try_init()
                    .map_err(|e| TracingInitError::SetGlobalDefault(e.to_string()))?;
            }
        }

        Ok(())
    }
}
