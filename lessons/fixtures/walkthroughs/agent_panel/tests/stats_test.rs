#[path = "../src/stats.rs"]
mod stats;

use stats::{calculate_mean, calculate_median, calculate_percentile};

#[test]
fn test_mean_calculation() {
    let numbers = vec![2.0, 4.0, 6.0, 8.0, 10.0];
    assert_eq!(calculate_mean(&numbers), Some(6.0));
    assert_eq!(calculate_mean(&[]), None);
}

#[test]
fn test_median_calculation() {
    let odd = vec![1.0, 5.0, 3.0];
    assert_eq!(calculate_median(&odd), Some(3.0));

    let even = vec![1.0, 2.0, 3.0, 4.0];
    assert_eq!(calculate_median(&even), Some(2.5));
    assert_eq!(calculate_median(&[]), None);
}

#[test]
fn test_percentile_calculation() {
    let numbers = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    assert_eq!(calculate_percentile(&numbers, 0.0), 10.0);
    assert_eq!(calculate_percentile(&numbers, 50.0), 30.0);
    assert_eq!(calculate_percentile(&numbers, 100.0), 50.0);
}
