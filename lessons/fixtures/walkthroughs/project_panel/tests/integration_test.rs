use workspace_service::utils::format::format_status;
use workspace_service::ServiceWorker;

#[test]
fn test_worker_execution() {
    let worker = ServiceWorker::new("test-node");
    let result = worker.execute();
    assert_eq!(format_status(&result), "[STATUS] Worker test-node completed dispatch");
}
