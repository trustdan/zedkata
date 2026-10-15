use workspace_service::utils::format::format_status;
use workspace_service::ServiceWorker;

fn main() {
    let worker = ServiceWorker::new("node-01");
    let report = worker.execute();
    println!("{}", format_status(&report));
}
