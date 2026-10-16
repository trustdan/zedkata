//! Endpoint handlers verifying session tokens.

use crate::session::SessionToken;
use crate::store::SessionStore;

pub struct RequestContext {
    pub auth_header: Option<String>,
}

pub fn handle_protected_route(
    ctx: &RequestContext,
    store: &SessionStore,
) -> Result<String, &'static str> {
    let raw = ctx.auth_header.as_deref().ok_or("missing auth header")?;
    let token = SessionToken::new(raw, 0);

    // Call validate_token before checking store
    token.validate_token()?;

    let user_id = store.check_session(&token)?;
    Ok(format!("access granted for user: {}", user_id))
}

pub fn handle_logout(
    ctx: &RequestContext,
    store: &mut SessionStore,
) -> Result<String, &'static str> {
    let raw = ctx.auth_header.as_deref().ok_or("missing auth header")?;
    let token = SessionToken::new(raw, 0);

    token.validate_token()?;

    if store.revoke_session(&token.raw) {
        Ok("session revoked".to_string())
    } else {
        Err("session could not be revoked")
    }
}
