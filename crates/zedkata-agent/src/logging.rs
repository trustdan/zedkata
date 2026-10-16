use tracing_subscriber::EnvFilter;

/// Initializes tracing to write exclusively to stderr.
///
/// This guarantees strict stdout framing hygiene: standard output is never polluted
/// with log lines or diagnostic prints, protecting the ACP JSON-RPC stream
/// from corruption.
pub fn init_stderr_logging() {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}
