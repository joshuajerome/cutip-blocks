//! Validate blocks — environment and path checks.

use std::net::IpAddr;
use std::path::Path;

use pyo3::prelude::*;

/// Check if a file or directory exists at the given path.
#[pyfunction]
#[pyo3(signature = (path))]
pub fn path_exists(path: &str) -> bool {
    let exists = Path::new(path).exists();
    eprintln!("[Validate] path_exists({path}): {exists}");
    exists
}

/// Check if an environment variable is set and non-empty.
#[pyfunction]
#[pyo3(signature = (name))]
pub fn env_var_set(name: &str) -> bool {
    let set = std::env::var(name).map_or(false, |v| !v.is_empty());
    eprintln!("[Validate] env_var_set({name}): {set}");
    set
}

/// Check if a string is a valid IPv4 or IPv6 address.
#[pyfunction]
#[pyo3(signature = (addr))]
pub fn ip_valid(addr: &str) -> bool {
    let valid = addr.trim().parse::<IpAddr>().is_ok();
    eprintln!("[Validate] ip_valid({addr}): {valid}");
    valid
}
