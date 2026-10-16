mod stats;

use stats::{calculate_mean, calculate_median, calculate_percentile};

fn main() {
    let dataset = vec![12.5, 4.0, 19.8, 7.2, 25.1, 14.3, 8.9];

    println!("=== Dataset Summary Statistics ===");
    println!("Count: {}", dataset.len());

    if let Some(mean) = calculate_mean(&dataset) {
        println!("Mean: {:.2}", mean);
    }

    if let Some(median) = calculate_median(&dataset) {
        println!("Median: {:.2}", median);
    }

    let p90 = calculate_percentile(&dataset, 90.0);
    println!("90th Percentile: {:.2}", p90);
}
