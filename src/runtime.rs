//! Shared tokio runtime for all cutip-blocks modules.
//!
//! A single multi-threaded tokio runtime is created on first use and shared
//! across SSH, container, HTTP, network, and service modules. This avoids
//! creating a new runtime per call and enables future connection sharing.

use std::sync::OnceLock;
use tokio::runtime::Runtime;
use pyo3::exceptions::PyRuntimeError;
use pyo3::PyResult;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or create the shared tokio runtime.
pub fn get() -> PyResult<&'static Runtime> {
    // OnceLock::get_or_try_init is unstable, so we use get_or_init
    // and panic on failure (runtime creation should never fail in practice)
    Ok(RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create tokio runtime")
    }))
}
