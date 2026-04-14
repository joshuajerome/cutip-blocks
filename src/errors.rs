//! Structured error types for cutip-blocks.
//!
//! Provides a taxonomy of errors that the cutip execution engine can use
//! to decide retry/abort/on_fail behavior:
//!   - ConnectionError  → retry (transient network issue)
//!   - TimeoutError     → retry (operation took too long)
//!   - AuthError        → abort (credentials are wrong, retrying won't help)
//!   - CommandFailed    → invoke on_fail (command ran but returned non-zero)
//!   - ValidationError  → abort (bad input, won't succeed on retry)

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

// ── Exception hierarchy ──
// All inherit from CutipBlocksError, which inherits from Python Exception.

create_exception!(cutip_blocks, CutipBlocksError, PyException, "Base error for all cutip-blocks operations.");
create_exception!(cutip_blocks, ConnectionError, CutipBlocksError, "Failed to establish or maintain a connection (SSH, Docker, HTTP). Retryable.");
create_exception!(cutip_blocks, TimeoutError, CutipBlocksError, "Operation exceeded its time limit. Retryable.");
create_exception!(cutip_blocks, AuthError, CutipBlocksError, "Authentication or authorization failed. Not retryable.");
create_exception!(cutip_blocks, CommandFailed, CutipBlocksError, "Command executed but returned a non-zero exit code.");
create_exception!(cutip_blocks, ValidationError, CutipBlocksError, "Input validation failed (bad path, missing env var, invalid IP). Not retryable.");

/// Register all exception types on the Python module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("CutipBlocksError", m.py().get_type::<CutipBlocksError>())?;
    m.add("ConnectionError", m.py().get_type::<ConnectionError>())?;
    m.add("TimeoutError", m.py().get_type::<TimeoutError>())?;
    m.add("AuthError", m.py().get_type::<AuthError>())?;
    m.add("CommandFailed", m.py().get_type::<CommandFailed>())?;
    m.add("ValidationError", m.py().get_type::<ValidationError>())?;
    Ok(())
}
