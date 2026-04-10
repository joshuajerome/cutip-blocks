//! Container blocks — lifecycle and exec operations via bollard (Docker/Podman API).

use std::collections::HashMap;
use std::path::Path;

use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions, WaitContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::BuildImageOptions;
use bollard::Docker;
use futures_util::StreamExt;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

/// A connection to the Docker/Podman daemon.
#[pyclass]
pub struct ContainerRuntime {
    client: Docker,
    runtime: Runtime,
}

impl ContainerRuntime {
    pub fn client(&self) -> &Docker {
        &self.client
    }
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
}

#[pymethods]
impl ContainerRuntime {
    /// Start a container by name.
    #[pyo3(signature = (name))]
    fn start(&self, py: Python<'_>, name: &str) -> PyResult<()> {
        eprintln!("[Container] Starting: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                client
                    .start_container(&name, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to start {name}: {e}")))?;
                Ok(())
            })
        })
    }

    /// Stop a running container.
    #[pyo3(signature = (name, timeout = 10))]
    fn stop(&self, py: Python<'_>, name: &str, timeout: i64) -> PyResult<()> {
        eprintln!("[Container] Stopping: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                client
                    .stop_container(
                        &name,
                        Some(StopContainerOptions { t: timeout }),
                    )
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to stop {name}: {e}")))?;
                Ok(())
            })
        })
    }

    /// Remove a container.
    #[pyo3(signature = (name, force = false))]
    fn remove(&self, py: Python<'_>, name: &str, force: bool) -> PyResult<()> {
        eprintln!("[Container] Removing: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                client
                    .remove_container(
                        &name,
                        Some(RemoveContainerOptions {
                            force,
                            ..Default::default()
                        }),
                    )
                    .await
                    .map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to remove {name}: {e}"))
                    })?;
                Ok(())
            })
        })
    }

    /// Execute a command inside a running container.
    #[pyo3(signature = (name, cmd))]
    fn exec(&self, py: Python<'_>, name: &str, cmd: &str) -> PyResult<ContainerExecResult> {
        eprintln!("[Container Exec] {name} :: {cmd}");
        let client = self.client.clone();
        let name = name.to_string();
        let cmd = cmd.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                let exec = client
                    .create_exec(
                        &name,
                        CreateExecOptions {
                            cmd: Some(vec!["bash".to_string(), "-c".to_string(), cmd]),
                            attach_stdout: Some(true),
                            attach_stderr: Some(true),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to create exec: {e}")))?;

                let mut stdout = Vec::new();
                let mut stderr = Vec::new();

                if let StartExecResults::Attached { mut output, .. } =
                    client.start_exec(&exec.id, None).await.map_err(|e| {
                        PyRuntimeError::new_err(format!("Failed to start exec: {e}"))
                    })?
                {
                    while let Some(msg) = output.next().await {
                        match msg {
                            Ok(bollard::container::LogOutput::StdOut { message }) => {
                                stdout.extend_from_slice(&message);
                            }
                            Ok(bollard::container::LogOutput::StdErr { message }) => {
                                stderr.extend_from_slice(&message);
                            }
                            _ => {}
                        }
                    }
                }

                let inspect = client
                    .inspect_exec(&exec.id)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to inspect exec: {e}")))?;
                let exit_code = inspect.exit_code.unwrap_or(-1) as i32;

                Ok(ContainerExecResult {
                    exit_code,
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                })
            })
        })
    }

    /// Pull an image from a registry.
    #[pyo3(signature = (image, tag = "latest"))]
    fn pull(&self, py: Python<'_>, image: &str, tag: &str) -> PyResult<()> {
        eprintln!("[Container] Pulling: {image}:{tag}");
        let client = self.client.clone();
        let image = image.to_string();
        let tag = tag.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                use bollard::image::CreateImageOptions;
                let opts = CreateImageOptions {
                    from_image: image.as_str(),
                    tag: tag.as_str(),
                    ..Default::default()
                };
                let mut stream = client.create_image(Some(opts), None, None);
                while let Some(result) = stream.next().await {
                    result.map_err(|e| {
                        PyRuntimeError::new_err(format!("Pull failed: {e}"))
                    })?;
                }
                eprintln!("[Container] Pulled: {image}:{tag}");
                Ok(())
            })
        })
    }

    /// Check if a container exists by name.
    #[pyo3(signature = (name))]
    fn exists(&self, py: Python<'_>, name: &str) -> PyResult<bool> {
        let client = self.client.clone();
        let name = name.to_string();
        py.allow_threads(|| {
            self.runtime.block_on(async {
                match client.inspect_container(&name, None).await {
                    Ok(_) => Ok(true),
                    Err(bollard::errors::Error::DockerResponseServerError {
                        status_code: 404,
                        ..
                    }) => Ok(false),
                    Err(e) => Err(PyRuntimeError::new_err(format!(
                        "Failed to inspect {name}: {e}"
                    ))),
                }
            })
        })
    }
}

/// Result of a container exec operation.
#[pyclass]
#[derive(Clone)]
pub struct ContainerExecResult {
    #[pyo3(get)]
    pub exit_code: i32,
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
}

#[pymethods]
impl ContainerExecResult {
    fn __repr__(&self) -> String {
        format!(
            "ContainerExecResult(exit_code={}, stdout={:?})",
            self.exit_code,
            self.stdout.chars().take(80).collect::<String>(),
        )
    }
}

/// Connect to the Docker/Podman daemon.
#[pyfunction]
#[pyo3(signature = (socket = None))]
pub fn container_connect(py: Python<'_>, socket: Option<&str>) -> PyResult<ContainerRuntime> {
    py.allow_threads(|| {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {e}")))?;

        let client = if let Some(socket_path) = socket {
            Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to connect to {socket_path}: {e}")))?
        } else {
            Docker::connect_with_local_defaults()
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to connect to Docker: {e}")))?
        };

        // Verify connection
        runtime.block_on(async {
            client.ping().await.map_err(|e| {
                PyRuntimeError::new_err(format!(
                    "Docker/Podman daemon not reachable: {e}. Is Docker Desktop running?"
                ))
            })
        })?;

        eprintln!("[Container] Connected to Docker/Podman daemon");
        Ok(ContainerRuntime { client, runtime })
    })
}
