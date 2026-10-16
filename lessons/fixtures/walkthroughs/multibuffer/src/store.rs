//! In-memory store for session tokens.

use crate::session::{SessionRecord, SessionStatus, SessionToken};
use std::collections::HashMap;

pub struct SessionStore {
    records: HashMap<String, SessionRecord>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn insert_session(&mut self, token: SessionToken, created_at: u64) {
        let key = token.raw.clone();
        let record = SessionRecord {
            token,
            status: SessionStatus::Active,
            created_at_epoch_s: created_at,
        };
        self.records.insert(key, record);
    }

    pub fn check_session(&self, token: &SessionToken) -> Result<u64, &'static str> {
        // First validate token format
        token.validate_token()?;

        match self.records.get(&token.raw) {
            Some(record) if record.status == SessionStatus::Active => Ok(record.token.user_id),
            Some(_) => Err("session is not active"),
            None => Err("session not found"),
        }
    }

    pub fn revoke_session(&mut self, raw_token: &str) -> bool {
        if let Some(record) = self.records.get_mut(raw_token) {
            record.status = SessionStatus::Revoked;
            true
        } else {
            false
        }
    }
}
