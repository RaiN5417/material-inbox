//! Pure domain model for Download Inbox.
//!
//! This crate MUST NOT depend on Tauri, SQLite, or any Windows API — see
//! docs/architecture.md. It only describes the shapes and state machine
//! that the rest of the workspace agrees on.

mod batch;
mod error;
mod file;
mod group;
mod operation;

pub use batch::{Batch, BatchStatus};
pub use error::{AppErrorCode, DomainError};
pub use file::{FileRecord, FileStatus};
pub use group::Group;
pub use operation::{Operation, OperationStatus, OperationType};
