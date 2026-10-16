mod handlers;
mod session;
mod store;

use handlers::{handle_protected_route, RequestContext};
use session::SessionToken;
use store::SessionStore;

fn main() {
    let mut store = SessionStore::new();

    let admin_token = SessionToken::new("zk_admin_9999", 42);
    // Explicit validation check
    if let Err(e) = admin_token.validate_token() {
        eprintln!("Admin token failed validation: {}", e);
        return;
    }

    store.insert_session(admin_token, 1700000000);

    let ctx = RequestContext {
        auth_header: Some("zk_admin_9999".to_string()),
    };

    match handle_protected_route(&ctx, &store) {
        Ok(msg) => println!("Response: {}", msg),
        Err(e) => eprintln!("Error: {}", e),
    }
}
