#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CalculationResult {
    pub operation: String,
    pub operand_a: i64,
    pub operand_b: i64,
    pub value: i64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StatsSummary {
    pub sample_count: usize,
    pub sum: i64,
    pub mean: i64,
    pub min: i64,
    pub max: i64,
}

pub fn execute_operation(op: &str, a: i64, b: i64) -> CalculationResult {
    let value = match op {
        "add" => a + b,
        "sub" => a - b,
        "mul" => a * b,
        "div" => {
            if b == 0 {
                0
            } else {
                a / b
            }
        }
        "mod" => {
            if b == 0 {
                0
            } else {
                a % b
            }
        }
        _ => 0,
    };

    CalculationResult {
        operation: op.to_string(),
        operand_a: a,
        operand_b: b,
        value,
    }
}

pub fn summarize(numbers: &[i64]) -> Option<StatsSummary> {
    if numbers.is_empty() {
        return None;
    }

    let mut sum = 0;
    let mut min = numbers[0];
    let mut max = numbers[0];

    for &num in numbers {
        sum += num;
        if num < min {
            min = num;
        }
        if num > max {
            max = num;
        }
    }

    let sample_count = numbers.len();
    let mean = sum / sample_count as i64;

    Some(StatsSummary {
        sample_count,
        sum,
        mean,
        min,
        max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_basic_arithmetic() {
        let add = execute_operation("add", 14, 6);
        assert_eq!(add.value, 20);

        let sub = execute_operation("sub", 50, 18);
        assert_eq!(sub.value, 32);

        let mul = execute_operation("mul", 7, 8);
        assert_eq!(mul.value, 56);

        let div = execute_operation("div", 100, 4);
        assert_eq!(div.value, 25);
    }

    #[test]
    fn test_division_by_zero_safety() {
        let div_zero = execute_operation("div", 42, 0);
        assert_eq!(div_zero.value, 0);
    }

    #[test]
    fn test_summarize_series() {
        let data = [10, 20, 30, 40, 50];
        let summary = summarize(&data).expect("data is non-empty");

        assert_eq!(summary.sample_count, 5);
        assert_eq!(summary.sum, 150);
        assert_eq!(summary.mean, 30);
        assert_eq!(summary.min, 10);
        assert_eq!(summary.max, 50);
    }
}
