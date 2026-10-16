use sample_calculator::math;

fn main() {
    println!("=== Zed Tasks & Debugger Sample Calculator ===");

    let step_1 = math::execute_operation("add", 40, 2);
    println!("Step 1: {} = {}", step_1.operation, step_1.value);

    let step_2 = math::execute_operation("mul", step_1.value, 3);
    println!("Step 2: {} = {}", step_2.operation, step_2.value);

    let series = [12, 45, 68, 23, 89, 34];
    if let Some(summary) = math::summarize(&series) {
        println!(
            "Summary: count={}, sum={}, mean={}, min={}, max={}",
            summary.sample_count, summary.sum, summary.mean, summary.min, summary.max
        );
    }

    println!("Calculator execution completed successfully.");
}
