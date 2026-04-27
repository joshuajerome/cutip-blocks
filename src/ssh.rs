//! SSH session management via russh.
//!
//! Provides `SSHSession` (PyO3 class) with `exec()` and `probe()` methods.
//! All SSH I/O happens in Rust via russh over a tokio runtime.
//! The Python GIL is released during network operations.

use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::{PyRuntimeError, PyTimeoutError};
use pyo3::prelude::*;
use regex::Regex;
use russh::client;
use russh::keys::PublicKey;
use russh::kex;
use russh::ChannelMsg;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;
use tokio::io::AsyncWriteExt;

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
        dim_log!(
            "[SSH] ssh {}@{} :: {}",
            inner.username, inner.host, display_cmd
        );

        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
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
        dim_log!("[SSH Probe] Authenticated as: {}", result.stdout.trim());
        Ok(result)
    }

    /// Close the SSH connection.
    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(inner) = self.inner.take() {
            let rt = crate::runtime::get()?;
            py.allow_threads(|| {
                rt.block_on(async {
                    let _ = inner
                        .handle
                        .disconnect(russh::Disconnect::ByApplication, "", "en")
                        .await;
                });
            });
            dim_log!("[SSH] Connection closed: {}", inner.host);
        }
        Ok(())
    }

    /// Open an interactive PTY shell session.
    ///
    /// Returns a ShellSession with expect/send methods for automating
    /// interactive CLI menus and prompts.
    fn shell(&mut self, py: Python<'_>) -> PyResult<ShellSession> {
        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("SSH session is closed"))?;

        let host = inner.host.clone();
        let redact = inner.redact.clone();

        dim_log!("[SSH Shell] Opening PTY session on {}", host);

        let rt = crate::runtime::get()?;
        let channel = py.allow_threads(|| {
            rt.block_on(async {
                let channel: russh::Channel<russh::client::Msg> = inner
                    .handle
                    .channel_open_session()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to open channel: {e}")))?;

                channel
                    .request_pty(false, "xterm", 80, 24, 0, 0, &[])
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to request PTY: {e}")))?;

                channel
                    .request_shell(false)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to request shell: {e}")))?;

                Ok::<_, PyErr>(channel)
            })
        })?;

        dim_log!("[SSH Shell] PTY session opened on {}", host);

        Ok(ShellSession {
            channel: Some(channel),
            buffer: String::new(),
            host,
            redact,
        })
    }

    /// Upload a local file to the remote host via SFTP.
    ///
    /// Streams the file in chunks (no full read into memory). Overwrites the
    /// destination if it exists.
    #[pyo3(signature = (local_path, remote_path))]
    fn upload(&mut self, py: Python<'_>, local_path: &str, remote_path: &str) -> PyResult<()> {
        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("SSH session is closed"))?;

        let host = inner.host.clone();
        let local = local_path.to_string();
        let remote = remote_path.to_string();

        dim_log!("[SFTP] upload {} -> {}:{}", local, host, remote);

        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                let channel = inner
                    .handle
                    .channel_open_session()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] upload to {}: failed to open channel: {}", host, e
                    )))?;

                channel
                    .request_subsystem(true, "sftp")
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] upload to {}: failed to request sftp subsystem: {}", host, e
                    )))?;

                let sftp = SftpSession::new(channel.into_stream())
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] upload to {}: failed to start sftp session: {}", host, e
                    )))?;

                let mut local_file = tokio::fs::File::open(&local).await.map_err(|e| {
                    PyRuntimeError::new_err(format!(
                        "[SFTP] upload {}: failed to open local file: {}", local, e
                    ))
                })?;

                let mut remote_file = sftp
                    .open_with_flags(
                        &remote,
                        OpenFlags::CREATE | OpenFlags::WRITE | OpenFlags::TRUNCATE,
                    )
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] upload to {}:{}: failed to open remote file: {}",
                        host, remote, e
                    )))?;

                let copied = tokio::io::copy(&mut local_file, &mut remote_file)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] upload {} -> {}:{}: copy error: {}",
                        local, host, remote, e
                    )))?;

                remote_file.shutdown().await.map_err(|e| {
                    PyRuntimeError::new_err(format!(
                        "[SFTP] upload to {}:{}: failed to flush remote file: {}",
                        host, remote, e
                    ))
                })?;

                // Close the SFTP session explicitly. Errors here are non-fatal
                // (the file is already written + flushed) but we log them.
                if let Err(e) = sftp.close().await {
                    dim_log!("[SFTP] upload to {}: warning closing session: {}", host, e);
                }

                dim_log!(
                    "[SFTP] upload complete: {} -> {}:{} ({} bytes)",
                    local, host, remote, copied
                );
                Ok(())
            })
        })
    }

    /// Download a file from the remote host via SFTP.
    ///
    /// Streams the file in chunks (no full read into memory). Overwrites the
    /// destination if it exists.
    #[pyo3(signature = (remote_path, local_path))]
    fn download(&mut self, py: Python<'_>, remote_path: &str, local_path: &str) -> PyResult<()> {
        let inner = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("SSH session is closed"))?;

        let host = inner.host.clone();
        let local = local_path.to_string();
        let remote = remote_path.to_string();

        dim_log!("[SFTP] download {}:{} -> {}", host, remote, local);

        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                let channel = inner
                    .handle
                    .channel_open_session()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] download from {}: failed to open channel: {}", host, e
                    )))?;

                channel
                    .request_subsystem(true, "sftp")
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] download from {}: failed to request sftp subsystem: {}",
                        host, e
                    )))?;

                let sftp = SftpSession::new(channel.into_stream())
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] download from {}: failed to start sftp session: {}",
                        host, e
                    )))?;

                let mut remote_file = sftp
                    .open_with_flags(&remote, OpenFlags::READ)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] download from {}:{}: failed to open remote file: {}",
                        host, remote, e
                    )))?;

                let mut local_file = tokio::fs::File::create(&local).await.map_err(|e| {
                    PyRuntimeError::new_err(format!(
                        "[SFTP] download to {}: failed to create local file: {}", local, e
                    ))
                })?;

                let copied = tokio::io::copy(&mut remote_file, &mut local_file)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!(
                        "[SFTP] download {}:{} -> {}: copy error: {}",
                        host, remote, local, e
                    )))?;

                local_file.shutdown().await.map_err(|e| {
                    PyRuntimeError::new_err(format!(
                        "[SFTP] download to {}: failed to flush local file: {}", local, e
                    ))
                })?;

                if let Err(e) = sftp.close().await {
                    dim_log!(
                        "[SFTP] download from {}: warning closing session: {}", host, e
                    );
                }

                dim_log!(
                    "[SFTP] download complete: {}:{} -> {} ({} bytes)",
                    host, remote, local, copied
                );
                Ok(())
            })
        })
    }
}

// ---------------------------------------------------------------------------
// ShellSession — interactive PTY with expect/send
// ---------------------------------------------------------------------------

/// Interactive PTY shell session with expect/send pattern matching.
///
/// Created via `SSHSession.shell()`. Automates interactive CLI menus
/// by sending input and waiting for expected output patterns.
#[pyclass]
pub struct ShellSession {
    channel: Option<russh::Channel<russh::client::Msg>>,
    buffer: String,
    host: String,
    redact: Vec<String>,
}

impl ShellSession {
    /// Read available data from the channel into the buffer, with timeout.
    fn read_into_buffer(
        channel: &mut russh::Channel<russh::client::Msg>,
        buffer: &mut String,
        timeout: Duration,
    ) -> Result<bool, PyErr> {
        let rt = crate::runtime::get()?;
        rt.block_on(async {
            match tokio::time::timeout(timeout, channel.wait()).await {
                Ok(Some(ChannelMsg::Data { data })) => {
                    buffer.push_str(&String::from_utf8_lossy(&data));
                    Ok(true)
                }
                Ok(Some(ChannelMsg::ExtendedData { data, .. })) => {
                    buffer.push_str(&String::from_utf8_lossy(&data));
                    Ok(true)
                }
                Ok(Some(ChannelMsg::Eof)) | Ok(None) => Ok(false),
                Ok(Some(_)) => Ok(true),
                Err(_) => Err(PyTimeoutError::new_err("Timed out waiting for data")),
            }
        })
    }
}

#[pymethods]
impl ShellSession {
    /// Wait until a regex pattern appears in the output.
    ///
    /// Args:
    ///     pattern: Regex pattern to match against accumulated output.
    ///     timeout: Seconds to wait before raising TimeoutError (default: 30).
    ///
    /// Returns:
    ///     The text matched by the pattern.
    #[pyo3(signature = (pattern, timeout = 30))]
    fn expect(&mut self, py: Python<'_>, pattern: &str, timeout: u64) -> PyResult<String> {
        let channel = self
            .channel
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Shell session is closed"))?;

        let re = Regex::new(pattern)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid regex: {e}")))?;

        let deadline = std::time::Instant::now() + Duration::from_secs(timeout);
        let read_chunk = Duration::from_millis(500);

        py.allow_threads(|| {
            loop {
                if let Some(m) = re.find(&self.buffer) {
                    let matched = m.as_str().to_string();
                    self.buffer = self.buffer[m.end()..].to_string();
                    dim_log!("[SSH Shell] expect({}) matched on {}", pattern, self.host);
                    return Ok(matched);
                }

                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    let display_buf = self.redact.iter().fold(self.buffer.clone(), |s, secret| {
                        s.replace(secret, "****")
                    });
                    return Err(PyTimeoutError::new_err(format!(
                        "expect('{}') timed out after {}s on {}. Buffer:\n{}",
                        pattern, timeout, self.host, display_buf
                    )));
                }

                let chunk_timeout = remaining.min(read_chunk);
                match Self::read_into_buffer(channel, &mut self.buffer, chunk_timeout) {
                    Ok(true) => continue,
                    Ok(false) => {
                        return Err(PyRuntimeError::new_err(format!(
                            "Channel closed while waiting for pattern '{}'",
                            pattern
                        )));
                    }
                    Err(_) => continue,
                }
            }
        })
    }

    /// Send text to the shell's stdin (no newline appended).
    #[pyo3(signature = (text))]
    fn send(&mut self, py: Python<'_>, text: &str) -> PyResult<()> {
        let channel = self
            .channel
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Shell session is closed"))?;

        let display = self.redact.iter().fold(text.to_string(), |s, secret| {
            s.replace(secret, "****")
        });
        dim_log!("[SSH Shell] send({:?}) to {}", display, self.host);

        let data = text.as_bytes().to_vec();
        let rt = crate::runtime::get()?;

        py.allow_threads(|| {
            rt.block_on(async {
                channel
                    .data(&data[..])
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to send data: {e}")))
            })
        })
    }

    /// Send text followed by a newline.
    #[pyo3(signature = (text))]
    fn send_line(&mut self, py: Python<'_>, text: &str) -> PyResult<()> {
        self.send(py, &format!("{}\n", text))
    }

    /// Read all output until a regex pattern is found.
    ///
    /// Unlike `expect()`, this returns ALL accumulated output up to
    /// and including the match (not just the matched portion).
    #[pyo3(signature = (pattern, timeout = 30))]
    fn read_until(&mut self, py: Python<'_>, pattern: &str, timeout: u64) -> PyResult<String> {
        let channel = self
            .channel
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Shell session is closed"))?;

        let re = Regex::new(pattern)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid regex: {e}")))?;

        let deadline = std::time::Instant::now() + Duration::from_secs(timeout);
        let read_chunk = Duration::from_millis(500);

        py.allow_threads(|| {
            loop {
                if let Some(m) = re.find(&self.buffer) {
                    let output = self.buffer[..m.end()].to_string();
                    self.buffer = self.buffer[m.end()..].to_string();
                    return Ok(output);
                }

                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(PyTimeoutError::new_err(format!(
                        "read_until('{}') timed out after {}s on {}",
                        pattern, timeout, self.host
                    )));
                }

                let chunk_timeout = remaining.min(read_chunk);
                match Self::read_into_buffer(channel, &mut self.buffer, chunk_timeout) {
                    Ok(true) => continue,
                    Ok(false) => {
                        return Err(PyRuntimeError::new_err(format!(
                            "Channel closed while waiting for pattern '{}'",
                            pattern
                        )));
                    }
                    Err(_) => continue,
                }
            }
        })
    }

    /// Read the next line of output.
    #[pyo3(signature = (timeout = 10))]
    fn read_line(&mut self, py: Python<'_>, timeout: u64) -> PyResult<String> {
        self.read_until(py, "\n", timeout)
    }

    /// Return any data currently in the buffer without waiting.
    fn peek(&self) -> String {
        self.buffer.clone()
    }

    /// Close the shell session.
    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        if let Some(channel) = self.channel.take() {
            let rt = crate::runtime::get()?;
            py.allow_threads(|| {
                rt.block_on(async {
                    let _ = channel.eof().await;
                    let _ = channel.close().await;
                });
            });
            dim_log!("[SSH Shell] Closed PTY session on {}", self.host);
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

    dim_log!("[SSH] Connecting to {}@{}:{}", username, host, port);

    let rt = crate::runtime::get()?;
    py.allow_threads(|| {
        let handle = rt.block_on(async {
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

            dim_log!("[SSH] Connected: {}@{}:{}", username, host, port);
            Ok(session)
        })?;

        Ok(SSHSession {
            inner: Some(SshInner {
                handle,
                host,
                username,
                redact,
            }),
        })
    })
}
