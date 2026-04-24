//! OS-native toast notifications.
//!
//! Uses `notify-rust` for cross-platform support:
//! - macOS: Notification Center
//! - Windows: WinRT toast notifications
//! - Linux: D-Bus (freedesktop)

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Send an OS-native toast notification.
///
/// Args:
///     title: Notification title.
///     message: Notification body text.
///     sound: Play a notification sound (default: true).
#[pyfunction]
#[pyo3(signature = (title, message, *, sound = true))]
pub fn send(title: &str, message: &str, sound: bool) -> PyResult<()> {
    dim_log!("[Notify] {}: {}", title, message);

    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(message);

    if sound {
        #[cfg(target_os = "macos")]
        notification.sound_name("Glass");

        #[cfg(target_os = "linux")]
        notification.sound_name("dialog-information");
    }

    notification
        .show()
        .map_err(|e| PyRuntimeError::new_err(format!("Notification failed: {e}")))?;

    Ok(())
}
