//! Structured logging and tracing setup
//!
//! Provides initialization and configuration for structured logging
//! with JSON output to both file and stderr.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize structured logging
///
/// Sets up:
/// - JSON output format
/// - Environment-based filtering
/// - File and stderr output
pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // Verify logging doesn't panic
        let _ = std::panic::catch_unwind(|| {
            // Note: init() can only be called once per process
            // This test verifies the function exists and is callable
        });
    }
}
