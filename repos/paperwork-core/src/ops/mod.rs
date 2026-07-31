//! Operations layer: stateless, path-explicit filesystem operations.
//!
//! Every operation takes an explicit file path. No workspace root, no init, no state.
//! Files are independent — no cross-references managed by the CLI.

pub mod contacts;
pub mod manifest;
pub mod notify;
pub mod profile;
pub mod thread;
