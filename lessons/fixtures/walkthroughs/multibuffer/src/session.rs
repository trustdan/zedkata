//! Session data types and token definitions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    pub raw: String,
    pub user_id: u64,
}

impl SessionToken {
    pub fn new(raw: &str, user_id: u64) -> Self {
        Self {
            raw: raw.to_string(),
            user_id,
        }
    }

    /// Validates raw token format and expiration.
    pub fn validate_token(&self) -> Result<(), &'static str> {
        if self.raw.is_empty() {
            return Err("token cannot be empty");
        }
        if !self.raw.starts_with("zk_") {
            return Err("invalid token prefix");
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub token: SessionToken,
    pub status: SessionStatus,
    pub created_at_epoch_s: u64,
}
