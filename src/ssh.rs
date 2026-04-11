//! SSH session management via russh.
//!
//! Provides `SSHSession` (PyO3 class) with `exec()` and `probe()` methods.
//! All SSH I/O happens in Rust via russh over a tokio runtime.
//! The Python GIL is released during network operations.

use std::borrow::Cow;
use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use russh::client;
use russh::keys::PublicKey;
use russh::kex;
use tokio::runtime::Runtime;

/// Result of a remote command execution.
#[pyclass]
#[derive(Clone)]
pub struct ExecResult {
    #[pyo3(get)]
    pub exit_code: i32,
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
}

#[pymethods]
impl ExecResult {
    fn __repr__(&self) -> String {
        format!(
            "ExecResult(exit_code={}, stdout={:?}, stderr={:?})",
            self.exit_code,
            self.stdout.chars().take(80).collect::<String>(),
            self.stderr.chars().take(80).collect::<String>(),
        )
    }
}

/// Handler for russh client events.
#[derive(Clone)]
struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all host keys (equivalent to paramiko AutoAddPolicy)
        Ok(true)
    }
}

/// Inner async SSH handle.
struct SshInner {
    handle: client::Handle<ClientHandler>,
    runtime: Runtime,
    host: String,
    username: String,
    redact: Vec<String>,
}

/// Persistent SSH session. Created via `ssh_connect()`.
///
/// Methods:
///   exec(cmd) — run a command, return ExecResult
///   probe()   — run `whoami`, verify connectivity
///   close()   — close the connection
#[pyclass]
pub struct SSHSession {
    inner: Option<SshInner>,
}

impl SSHSession {
    /// Execute a remote command (callable from other Rust modules).
    pub fn run_cmd(&mut self, py: Python<'_>, cmd: &str) -> PyResult<ExecResult> {
        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("SSH session is closed"))?;

        let cmd_owned = cmd.to_string();

        let display_cmd = inner
            .redact
            .iter()
            .fold(cmd_owned.clone(), |s, secret| s.replace(secret, "****"));
        eprintln!(
            "[SSH] ssh {}@{} :: {}",
            inner.username, inner.host, display_cmd
        );

        py.allow_threads(|| {
            inner.runtime.block_on(async {
                let mut channel =
                    inner.handle.channel_open_session().await.map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to open channel: {e}"))
                    })?;

                channel
                    .exec(true, cmd_owned.as_bytes())
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to exec: {e}")))?;

                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                let mut exit_code: i32 = -1;
                let mut got_eof = false;

                loop {
                    match channel.wait().await {
                        Some(russh::ChannelMsg::Data { data }) => {
                            stdout.extend_from_slice(&data);
                        }
                        Some(russh::ChannelMsg::ExtendedData { data, ext }) => {
                            if ext == 1 {
                                stderr.extend_from_slice(&data);
                            }
                        }
                        Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                            exit_code = exit_status as i32;
                            if got_eof {
                                break;
                            }
                        }
                        Some(russh::ChannelMsg::Eof) => {
                            got_eof = true;
                            // Don't break yet — ExitStatus may arrive after Eof
                            if exit_code >= 0 {
                                break;
                            }
                        }
                        None => break,
                        _ => {}
                    }
                }

                Ok(ExecResult {
                    exit_code,
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                })
            })
        })
    }
}

#[pymethods]
impl SSHSession {
    /// Execute a remote command.
    #[pyo3(signature = (cmd))]
    fn exec(&mut self, py: Python<'_>, cmd: &str) -> PyResult<ExecResult> {
        self.run_cmd(py, cmd)
    }

    /// Run `whoami` to verify SSH connectivity.
    fn probe(&mut self, py: Python<'_>) -> PyResult<ExecResult> {
        let result = self.run_cmd(py, "whoami")?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "SSH probe failed: {}",
                result.stderr.trim()
            )));
        }
        eprintln!("[SSH Probe] Authenticated as: {}", result.stdout.trim());
        Ok(result)
    }

    /// Close the SSH connection.
    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(inner) = self.inner.take() {
            py.allow_threads(|| {
                inner.runtime.block_on(async {
                    let _ = inner
                        .handle
                        .disconnect(russh::Disconnect::ByApplication, "", "en")
                        .await;
                });
            });
            eprintln!("[SSH] Connection closed: {}", inner.host);
        }
        Ok(())
    }
}

/// Open an SSH connection.
#[pyfunction]
#[pyo3(signature = (*, host, username, password, port = 22))]
pub fn ssh_connect(
    py: Python<'_>,
    host: &str,
    username: &str,
    password: &str,
    port: u16,
) -> PyResult<SSHSession> {
    let host = host.to_string();
    let username = username.to_string();
    let password = password.to_string();
    let redact = vec![password.clone()];

    eprintln!("[SSH] Connecting to {}@{}:{}", username, host, port);

    py.allow_threads(|| {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {e}")))?;

        let handle = runtime.block_on(async {
            let mut config = client::Config::default();
            // Add ECDH NIST P-256/384/521 to the default kex list
            // so we can connect to servers that only support these
            let mut kex_list = config.preferred.kex.to_vec();
            for algo in &[kex::ECDH_SHA2_NISTP256, kex::ECDH_SHA2_NISTP384, kex::ECDH_SHA2_NISTP521] {
                if !kex_list.contains(algo) {
                    kex_list.push(*algo);
                }
            }
            config.preferred.kex = Cow::Owned(kex_list);
            let config = Arc::new(config);

            let handler = ClientHandler;
            let mut session = client::connect(config, (host.as_str(), port), handler)
                .await
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("SSH connection failed to {host}:{port}: {e}"))
                })?;

            let auth_result = session
                .authenticate_password(&username, &password)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("SSH auth error: {e}")))?;

            if !auth_result.success() {
                return Err(PyRuntimeError::new_err(format!(
                    "SSH authentication failed for {username}@{host}"
                )));
            }

            eprintln!("[SSH] Connected: {}@{}:{}", username, host, port);
            Ok(session)
        })?;

        Ok(SSHSession {
            inner: Some(SshInner {
                handle,
                runtime,
                host,
                username,
                redact,
            }),
        })
    })
}
