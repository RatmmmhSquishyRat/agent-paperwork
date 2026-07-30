//! Notification operations: push, list unread, ack (move to history).

use std::fs;
use std::path::Path;

use crate::error::{PaperworkError, Result};
use crate::format::notification::{parse_notifications, serialize_notifications};
use crate::layout;
use crate::Notification;

/// Push a notification to an agent's unread queue.
pub fn push_notify(root: &Path, target: &str, notification: &Notification) -> Result<()> {
    layout::ensure_initialized(root)?;

    // Ensure notification directory exists
    let agent_dir = layout::notification_agent_dir(root, target);
    fs::create_dir_all(&agent_dir).map_err(|e| PaperworkError::IoContext {
        path: agent_dir.clone(),
        source: e,
    })?;

    let unread_path = layout::unread_path(root, target);

    // Read existing notifications (or start empty)
    let mut notifications = if unread_path.exists() {
        let content = fs::read_to_string(&unread_path).map_err(|e| PaperworkError::IoContext {
            path: unread_path.clone(),
            source: e,
        })?;
        parse_notifications(&content)?
    } else {
        Vec::new()
    };

    // Add new notification
    notifications.push(notification.clone());

    // Write back
    let serialized = serialize_notifications(target, &notifications);
    fs::write(&unread_path, serialized).map_err(|e| PaperworkError::IoContext {
        path: unread_path.clone(),
        source: e,
    })?;

    Ok(())
}

/// List unread notifications for an agent.
pub fn list_unread(root: &Path, agent: &str) -> Result<Vec<Notification>> {
    layout::ensure_initialized(root)?;

    let unread_path = layout::unread_path(root, agent);

    if !unread_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&unread_path).map_err(|e| PaperworkError::IoContext {
        path: unread_path.clone(),
        source: e,
    })?;

    parse_notifications(&content)
}

/// Acknowledge all unread notifications: move from unread to history.
///
/// Returns the notifications that were acknowledged.
pub fn ack_notify(root: &Path, agent: &str) -> Result<Vec<Notification>> {
    layout::ensure_initialized(root)?;

    let unread_path = layout::unread_path(root, agent);
    let history_path = layout::history_path(root, agent);

    // Ensure notification directory exists
    let agent_dir = layout::notification_agent_dir(root, agent);
    fs::create_dir_all(&agent_dir).map_err(|e| PaperworkError::IoContext {
        path: agent_dir.clone(),
        source: e,
    })?;

    // Read unread notifications
    let unread = if unread_path.exists() {
        let content = fs::read_to_string(&unread_path).map_err(|e| PaperworkError::IoContext {
            path: unread_path.clone(),
            source: e,
        })?;
        parse_notifications(&content)?
    } else {
        Vec::new()
    };

    if unread.is_empty() {
        return Ok(Vec::new());
    }

    // Read existing history
    let mut history = if history_path.exists() {
        let content = fs::read_to_string(&history_path).map_err(|e| PaperworkError::IoContext {
            path: history_path.clone(),
            source: e,
        })?;
        parse_notifications(&content)?
    } else {
        Vec::new()
    };

    // Append unread to history
    history.extend(unread.clone());

    // Write history
    let history_serialized = serialize_notifications(agent, &history);
    fs::write(&history_path, history_serialized).map_err(|e| PaperworkError::IoContext {
        path: history_path.clone(),
        source: e,
    })?;

    // Clear unread (write empty)
    let empty_serialized = serialize_notifications(agent, &[]);
    fs::write(&unread_path, empty_serialized).map_err(|e| PaperworkError::IoContext {
        path: unread_path.clone(),
        source: e,
    })?;

    Ok(unread)
}

/// List notification history for an agent.
pub fn list_history(root: &Path, agent: &str) -> Result<Vec<Notification>> {
    layout::ensure_initialized(root)?;

    let history_path = layout::history_path(root, agent);

    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&history_path).map_err(|e| PaperworkError::IoContext {
        path: history_path.clone(),
        source: e,
    })?;

    parse_notifications(&content)
}
