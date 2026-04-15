//! Service management blocks — systemd service control.
//!
//! Provides start, stop, restart, enable, disable, and status for systemd services.

use std::process::Command;

use pyo3::prelude::*;

use crate::errors;
use crate::ssh::ExecResult;

fn systemctl(py: Python<'_>, action: &str, service: &str) -> PyResult<ExecResult> {
    eprintln!("[Svc] systemctl {action} {service}");

    let action = action.to_string();
    let service = service.to_string();

    py.allow_threads(|| {
        let output = Command::new("systemctl")
            .arg(&action)
            .arg(&service)
            .output()
            .map_err(|e| errors::CommandFailed::new_err(format!(
                "Failed to run systemctl {action} {service}: {e}"
            )))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if exit_code != 0 && action != "status" {
            return Err(errors::CommandFailed::new_err(format!(
                "systemctl {action} {service} failed: {}",
                stderr.trim()
            )));
        }

        Ok(ExecResult { exit_code, stdout, stderr })
    })
}

/// Start a systemd service.
#[pyfunction]
#[pyo3(signature = (service))]
pub fn start(py: Python<'_>, service: &str) -> PyResult<()> {
    systemctl(py, "start", service)?;
    Ok(())
}

/// Stop a systemd service.
#[pyfunction]
#[pyo3(signature = (service))]
pub fn stop(py: Python<'_>, service: &str) -> PyResult<()> {
    systemctl(py, "stop", service)?;
    Ok(())
}

/// Restart a systemd service.
#[pyfunction]
#[pyo3(signature = (service))]
pub fn restart(py: Python<'_>, service: &str) -> PyResult<()> {
    systemctl(py, "restart", service)?;
    Ok(())
}

/// Enable a systemd service (start on boot).
#[pyfunction]
#[pyo3(signature = (service))]
pub fn enable(py: Python<'_>, service: &str) -> PyResult<()> {
    systemctl(py, "enable", service)?;
    Ok(())
}

/// Disable a systemd service.
#[pyfunction]
#[pyo3(signature = (service))]
pub fn disable(py: Python<'_>, service: &str) -> PyResult<()> {
    systemctl(py, "disable", service)?;
    Ok(())
}

/// Check if a systemd service is active (running).
#[pyfunction]
#[pyo3(signature = (service))]
pub fn is_active(py: Python<'_>, service: &str) -> PyResult<bool> {
    let result = systemctl(py, "is-active", service)?;
    Ok(result.stdout.trim() == "active")
}

/// Get the status of a systemd service.
#[pyfunction]
#[pyo3(signature = (service))]
pub fn status(py: Python<'_>, service: &str) -> PyResult<ExecResult> {
    systemctl(py, "status", service)
}
