//! Statistical utility module for the kata-agent demo workspace.

/// Calculates the arithmetic mean of a slice of floats.
/// Returns None if the slice is empty.
pub fn calculate_mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(sum / values.len() as f64)
}

/// Calculates the median of a slice of floats.
/// Returns None if the slice is empty.
pub fn calculate_median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

/// Computes the p-th percentile (0.0 to 100.0) from a slice of floats.
///
/// NOTE: Demonstrates an unrefactored target for the AI agent walkthrough.
/// Currently panics on empty slices and clamps p between 0 and 100.
pub fn calculate_percentile(values: &[f64], p: f64) -> f64 {
    assert!(!values.is_empty(), "values slice must not be empty");
    let clamped_p = p.clamp(0.0, 100.0);
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let rank = (clamped_p / 100.0) * (sorted.len() - 1) as f64;
    let lower_index = rank.floor() as usize;
    let upper_index = rank.ceil() as usize;
    let weight = rank - lower_index as f64;

    sorted[lower_index] * (1.0 - weight) + sorted[upper_index] * weight
}
