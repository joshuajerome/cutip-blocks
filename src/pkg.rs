//! Package management blocks — install/remove system packages.
//!
//! Auto-detects the package manager (apt, dnf, yum, apk) and runs
//! the appropriate install/remove commands via subprocess.

use std::process::Command;

use pyo3::prelude::*;

use crate::errors;

/// Detect the system package manager.
fn detect_pkg_manager() -> PyResult<&'static str> {
    for cmd in &["apt-get", "dnf", "yum", "apk"] {
        let check = if cfg!(target_os = "windows") {
            Command::new("where").arg(cmd).output()
        } else {
            Command::new("which").arg(cmd).output()
        };

        if let Ok(output) = check {
            if output.status.success() {
                return Ok(cmd);
            }
        }
    }

    Err(errors::ValidationError::new_err(
        "No supported package manager found (apt-get, dnf, yum, apk)"
    ))
}

/// Build the install command for the detected package manager.
fn install_cmd(manager: &str, packages: &[String]) -> Vec<String> {
    let pkgs = packages.join(" ");
    match manager {
        "apt-get" => vec![
            "sh".into(), "-c".into(),
            format!("DEBIAN_FRONTEND=noninteractive apt-get install -y {pkgs}"),
        ],
        "dnf" => vec!["dnf".into(), "install".into(), "-y".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        "yum" => vec!["yum".into(), "install".into(), "-y".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        "apk" => vec!["apk".into(), "add".into(), "--no-cache".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        _ => vec!["sh".into(), "-c".into(), format!("{manager} install -y {pkgs}")],
    }
}

/// Build the remove command for the detected package manager.
fn remove_cmd(manager: &str, packages: &[String]) -> Vec<String> {
    let pkgs = packages.join(" ");
    match manager {
        "apt-get" => vec![
            "sh".into(), "-c".into(),
            format!("DEBIAN_FRONTEND=noninteractive apt-get remove -y {pkgs}"),
        ],
        "dnf" => vec!["dnf".into(), "remove".into(), "-y".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        "yum" => vec!["yum".into(), "remove".into(), "-y".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        "apk" => vec!["apk".into(), "del".into()].into_iter()
            .chain(packages.iter().cloned()).collect(),
        _ => vec!["sh".into(), "-c".into(), format!("{manager} remove -y {pkgs}")],
    }
}

/// Install system packages. Auto-detects apt/dnf/yum/apk.
///
/// Usage: `pkg.install(["git", "curl", "wget"])`
#[pyfunction]
#[pyo3(signature = (packages))]
pub fn install(py: Python<'_>, packages: Vec<String>) -> PyResult<()> {
    let manager = detect_pkg_manager()?;
    dim_log!("[Pkg] Installing via {manager}: {}", packages.join(", "));

    let cmd = install_cmd(manager, &packages);

    py.allow_threads(|| {
        let output = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| errors::CommandFailed::new_err(format!("Failed to run {manager}: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(errors::CommandFailed::new_err(format!(
                "Package install failed: {}",
                stderr.trim()
            )));
        }

        dim_log!("[Pkg] Installed: {}", packages.join(", "));
        Ok(())
    })
}

/// Remove system packages. Auto-detects apt/dnf/yum/apk.
///
/// Usage: `pkg.remove(["vim", "nano"])`
#[pyfunction]
#[pyo3(signature = (packages))]
pub fn remove(py: Python<'_>, packages: Vec<String>) -> PyResult<()> {
    let manager = detect_pkg_manager()?;
    dim_log!("[Pkg] Removing via {manager}: {}", packages.join(", "));

    let cmd = remove_cmd(manager, &packages);

    py.allow_threads(|| {
        let output = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| errors::CommandFailed::new_err(format!("Failed to run {manager}: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(errors::CommandFailed::new_err(format!(
                "Package remove failed: {}",
                stderr.trim()
            )));
        }

        dim_log!("[Pkg] Removed: {}", packages.join(", "));
        Ok(())
    })
}

/// Update the package index. Auto-detects apt/dnf/yum/apk.
#[pyfunction]
#[pyo3(signature = ())]
pub fn update(py: Python<'_>) -> PyResult<()> {
    let manager = detect_pkg_manager()?;
    dim_log!("[Pkg] Updating package index via {manager}");

    let cmd: Vec<String> = match manager {
        "apt-get" => vec!["sh".into(), "-c".into(), "DEBIAN_FRONTEND=noninteractive apt-get update".into()],
        "dnf" => vec!["dnf".into(), "makecache".into()],
        "yum" => vec!["yum".into(), "makecache".into()],
        "apk" => vec!["apk".into(), "update".into()],
        _ => vec!["sh".into(), "-c".into(), format!("{manager} update")],
    };

    py.allow_threads(|| {
        let output = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| errors::CommandFailed::new_err(format!("Failed to update: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(errors::CommandFailed::new_err(format!(
                "Package update failed: {}",
                stderr.trim()
            )));
        }
        Ok(())
    })
}
