//! Local shell command execution.
//!
//! Provides `run()` to execute commands on the local host via `std::process::Command`.
//! Returns `ExecResult` (same type as SSH exec) for consistent interface.
//! The Python GIL is released during command execution.

use std::collections::HashMap;
use std::process::Command;

use pyo3::prelude::*;

use crate::errors;
use crate::ssh::ExecResult;

/// Execute a shell command on the local host.
///
/// Returns an `ExecResult` with exit_code, stdout, and stderr.
/// Raises `CommandFailed` if the command returns a non-zero exit code
/// and `check` is True (default).
///
/// # Arguments
/// * `cmd` — The command string to execute (passed to `sh -c` on Unix, `cmd /C` on Windows).
/// * `cwd` — Optional working directory.
/// * `env` — Optional environment variables (merged with current env).
/// * `check` — If true (default), raise `CommandFailed` on non-zero exit. If false, return the result regardless.
#[pyfunction]
#[pyo3(signature = (cmd, *, cwd=None, env=None, check=true))]
pub fn run(
    py: Python<'_>,
    cmd: &str,
    cwd: Option<&str>,
    env: Option<HashMap<String, String>>,
    check: bool,
) -> PyResult<ExecResult> {
    let cmd_display = if cmd.len() > 120 {
        format!("{}...", &cmd[..120])
    } else {
        cmd.to_string()
    };
    eprintln!("[Shell] {cmd_display}");

    let cmd_owned = cmd.to_string();
    let cwd_owned = cwd.map(|s| s.to_string());
    let env_owned = env;

    py.allow_threads(|| {
        let mut command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&cmd_owned);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&cmd_owned);
            c
        };

        if let Some(ref dir) = cwd_owned {
            command.current_dir(dir);
        }

        if let Some(ref env_vars) = env_owned {
            for (key, value) in env_vars {
                command.env(key, value);
            }
        }

        let output = command
            .output()
            .map_err(|e| errors::ConnectionError::new_err(format!("Failed to execute command: {e}")))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if check && exit_code != 0 {
            let err_msg = if stderr.trim().is_empty() {
                format!("Command failed with exit code {exit_code}: {cmd_owned}")
            } else {
                format!(
                    "Command failed with exit code {exit_code}: {}",
                    stderr.trim()
                )
            };
            return Err(errors::CommandFailed::new_err(err_msg));
        }

        Ok(ExecResult {
            exit_code,
            stdout,
            stderr,
        })
    })
}
