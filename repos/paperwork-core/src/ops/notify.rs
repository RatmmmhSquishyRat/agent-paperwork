//! Notification operations: push, read — all path-explicit.
//!
//! A notification file is an append-only log adjacent to a profile
//! (e.g., `alice.notify.md`). No workspace, no unread/history split.

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::notification::{parse_notifications, serialize_notifications};
use crate::Notification;

/// Push (append) a notification to the file at `path`.
///
/// Creates the file (and parent dirs) if it doesn't exist.
/// The `name` parameter is used for the H1 heading in the file.
pub fn notify_push(path: &Path, name: &str, notification: &Notification) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| PaperworkError::IoContext {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Read existing notifications (or start empty)
    let mut notifications = if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
            path: path.to_path_buf(),
            source: e,
        })?;
        parse_notifications(&content)?
    } else {
        Vec::new()
    };

    notifications.push(notification.clone());

    let serialized = serialize_notifications(name, &notifications);
    fs::write(path, serialized).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Read all notifications from the file at `path`.
pub fn notify_read(path: &Path) -> Result<Vec<Notification>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path).map_err(|e| PaperworkError::IoContext {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_notifications(&content)
}
