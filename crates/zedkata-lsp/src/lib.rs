//! Language server observer for zedkata drills in Zed.
//!
//! Tracks active `.kata` buffers, records fine-grained incremental edits from
//! `textDocument/didChange`, coordinates session persistence, and emits inline diagnostics.

pub mod capture;
pub mod codec;
pub mod document;
pub mod error;
pub mod feedback;
pub mod protocol;
pub mod server;

pub use capture::Capture;
pub use codec::{write_message, write_message_sync, MessageReader};
pub use document::{
    byte_offset_to_position, position_to_byte_offset, DocumentState, DocumentTracker,
};
pub use error::{DocumentError, LspError};
pub use feedback::{paths_match, uri_to_path, FeedbackEngine};
pub use protocol::*;
pub use server::{
    run_server, run_server_with_engine, LspServer, MessageOutput, ServerLifecycleState,
};
