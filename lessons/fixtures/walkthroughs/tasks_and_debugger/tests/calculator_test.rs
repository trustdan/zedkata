use sample_calculator::math;

#[test]
fn test_integration_calculator_pipeline() {
    let initial = math::execute_operation("add", 100, 25);
    let multiplied = math::execute_operation("mul", initial.value, 2);
    let final_val = math::execute_operation("sub", multiplied.value, 50);

    assert_eq!(initial.value, 125);
    assert_eq!(multiplied.value, 250);
    assert_eq!(final_val.value, 200);
}
