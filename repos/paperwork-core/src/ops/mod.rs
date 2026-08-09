//! Operations layer: stateless, path-explicit filesystem operations.
//!
//! Every operation takes an explicit file path. No workspace root, no init, no state.
//! Files are independent — no cross-references managed by the CLI.

pub mod contacts;
// Internal helper: only consumed by core ops modules; kept out of the
// public semver surface until a real cross-crate need appears (impact
// review Oscar m-2).
pub(crate) mod lock;
pub mod manifest;
pub mod profile;
pub mod thread;
