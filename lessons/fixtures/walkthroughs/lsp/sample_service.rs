//! Sample service fixture for Zed LSP-backed action walkthroughs.
//!
//! Demonstrates:
//! - Multi-site symbol rename (`c d`)
//! - Quick-fix code actions (`g .`)
//! - Inline diagnostic errors and navigation (`g ]` / `g [`, `] d` / `[ d`)
//! - Documentation hover (`g h`)
//! - Definition jumping (`g d`, `<ctrl-w> g d`) and references (`g A`)
//! - Diagnostics panel (`:clist`)

use std::collections::HashMap;

/// Severity level for telemetry events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLevel {
    Info,
    Warning,
    Critical,
}

/// A structured metric recorded by the telemetry pipeline.
#[derive(Debug, Clone)]
pub struct TelemetryMetric {
    pub name: String,
    pub value: f64,
    pub level: MetricLevel,
}

/// Service responsible for accumulating and flushing application telemetry.
pub struct TelemetryClient {
    metrics: Vec<TelemetryMetric>,
    tags: HashMap<String, String>,
}

impl TelemetryClient {
    /// Construct a new client initialized with an empty buffer and default tags.
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            tags: HashMap::new(),
        }
    }

    /// Attach a global tag key-value pair to all subsequent events.
    pub fn add_tag(&mut self, key: &str, value: &str) {
        self.tags.insert(key.to_string(), value.to_string());
    }

    /// Record a single telemetry metric into the local dispatch buffer.
    pub fn record_metric(&mut self, name: &str, value: f64, level: MetricLevel) {
        let metric = TelemetryMetric {
            name: name.to_string(),
            value,
            level,
        };
        self.metrics.push(metric);
    }

    /// Compute the arithmetic mean of all recorded metric values.
    pub fn compute_average(&self) -> f64 {
        if self.metrics.is_empty() {
            return 0.0;
        }
        let total: f64 = self.metrics.iter().map(|m| m.value).sum();
        total / (self.metrics.len() as f64)
    }

    /// Reset internal metric storage after dispatch.
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

/// Helper function to build a standard production telemetry dispatcher.
pub fn initialize_client() -> TelemetryClient {
    let mut client = TelemetryClient::new();
    client.add_tag("environment", "production");
    client.add_tag("region", "us-east-1");

    // Sample recordings
    client.record_metric("cpu_usage_pct", 42.5, MetricLevel::Info);
    client.record_metric("memory_usage_mb", 1024.0, MetricLevel::Info);
    client.record_metric("disk_io_latency_ms", 12.8, MetricLevel::Warning);

    client
}
