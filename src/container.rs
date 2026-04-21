//! Container blocks — lifecycle and exec operations via bollard (Docker/Podman API).

use std::collections::HashMap;
use std::path::Path;

use bollard::container::{
    Config as ContainerCreateConfig, CreateContainerOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::BuildImageOptions;
use bollard::models::{HostConfig, Mount, MountTypeEnum, PortBinding};
use bollard::Docker;
use futures_util::StreamExt;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::errors;

/// A connection to the Docker/Podman daemon.
#[pyclass]
pub struct ContainerRuntime {
    client: Docker,
}

impl ContainerRuntime {
    pub fn client(&self) -> &Docker {
        &self.client
    }
}

#[pymethods]
impl ContainerRuntime {
    /// Build an image from a Dockerfile.
    ///
    /// Streams build output to stderr. Returns the image ID.
    #[pyo3(signature = (*, context, dockerfile, tag, build_args = None, network_mode = None, timeout = 300))]
    fn build(
        &self,
        py: Python<'_>,
        context: &str,
        dockerfile: &str,
        tag: &str,
        build_args: Option<HashMap<String, String>>,
        network_mode: Option<&str>,
        timeout: u64,
    ) -> PyResult<String> {
        dim_log!("[Container] Building: {tag} from {context}/{dockerfile} (timeout: {timeout}s)");

        let context_path = Path::new(context);
        if !context_path.is_dir() {
            return Err(errors::ValidationError::new_err(format!(
                "Build context not found: {context}"
            )));
        }

        let df_path = context_path.join(dockerfile);
        if !df_path.is_file() {
            return Err(errors::ValidationError::new_err(format!(
                "Dockerfile not found: {}", df_path.display()
            )));
        }

        // Create tar archive of context directory
        let tar_bytes = create_tar_archive(context_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create build context archive: {e}")))?;

        let tag_owned = tag.to_string();
        let dockerfile_owned = dockerfile.to_string();
        let build_args_owned = build_args.unwrap_or_default();
        let network_owned = network_mode.map(|s| s.to_string());
        let client = self.client.clone();

        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                let mut opts = BuildImageOptions {
                    t: tag_owned.as_str(),
                    dockerfile: dockerfile_owned.as_str(),
                    rm: true,
                    ..Default::default()
                };

                if let Some(ref net) = network_owned {
                    opts.networkmode = net.as_str();
                }

                // Convert build args to the format bollard expects
                let args_str: HashMap<&str, &str> = build_args_owned
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                opts.buildargs = args_str;

                // Pass empty credentials map to avoid Podman X-Registry-Config parse error on Windows
                let empty_creds: HashMap<String, bollard::auth::DockerCredentials> = HashMap::new();
                let mut stream = client.build_image(opts, Some(empty_creds), Some(tar_bytes.into()));
                let mut image_id = String::new();
                let idle_timeout = std::time::Duration::from_secs(timeout);

                loop {
                    let result = match tokio::time::timeout(idle_timeout, stream.next()).await {
                        Ok(Some(r)) => r,
                        Ok(None) => break, // stream ended
                        Err(_) => return Err(errors::TimeoutError::new_err(format!(
                            "Build timed out — no output for {timeout}s. Increase with timeout= parameter."
                        ))),
                    };
                    match result {
                        Ok(output) => {
                            if let Some(ref stream_str) = output.stream {
                                let line = stream_str.trim_end();
                                if !line.is_empty() {
                                    dim_log!("  {line}");
                                }
                            }
                            if let Some(ref id) = output.aux {
                                if let Some(ref id_str) = id.id {
                                    image_id = id_str.clone();
                                }
                            }
                            if let Some(ref err) = output.error {
                                return Err(errors::CommandFailed::new_err(format!(
                                    "Build failed: {err}"
                                )));
                            }
                        }
                        Err(e) => {
                            return Err(errors::CommandFailed::new_err(format!(
                                "Build failed: {e}"
                            )));
                        }
                    }
                }

                dim_log!("[Container] Built: {tag_owned}");
                Ok(image_id)
            })
        })
    }

    /// Create a container from an image.
    ///
    /// Returns the container ID. The container is created but not started.
    #[pyo3(signature = (
        *,
        name,
        image,
        network_mode = None,
        privileged = false,
        hostname = None,
        workdir = None,
        command = None,
        environment = None,
        mounts = None,
        labels = None,
        ports = None,
        restart_policy = None
    ))]
    #[allow(clippy::too_many_arguments)]
    fn create(
        &self,
        py: Python<'_>,
        name: &str,
        image: &str,
        network_mode: Option<&str>,
        privileged: bool,
        hostname: Option<&str>,
        workdir: Option<&str>,
        command: Option<&str>,
        environment: Option<HashMap<String, String>>,
        mounts: Option<&Bound<'_, PyList>>,
        labels: Option<HashMap<String, String>>,
        ports: Option<HashMap<String, String>>,
        restart_policy: Option<&str>,
    ) -> PyResult<String> {
        dim_log!("[Container] Creating: {name} from {image}");

        let name_owned = name.to_string();
        let image_owned = image.to_string();
        let network_mode_owned = network_mode.map(|s| s.to_string());
        let hostname_owned = hostname.map(|s| s.to_string());
        let workdir_owned = workdir.map(|s| s.to_string());
        let env_owned = environment.unwrap_or_default();
        let labels_owned = labels.unwrap_or_default();
        let ports_owned = ports.unwrap_or_default();
        let restart_owned = restart_policy.map(|s| s.to_string());

        // Parse command string into vec
        let cmd: Option<Vec<String>> = command.map(|c| {
            vec!["/bin/sh".to_string(), "-c".to_string(), c.to_string()]
        });

        // Convert environment to Docker format: ["KEY=value", ...]
        let env_vec: Vec<String> = env_owned
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        // Parse mounts from Python list of dicts
        let docker_mounts = parse_mounts(mounts)?;

        // Parse port bindings
        let (exposed_ports, port_bindings) = parse_ports(&ports_owned);

        // Build restart policy
        let restart = restart_owned.map(|policy| {
            bollard::models::RestartPolicy {
                name: Some(match policy.as_str() {
                    "always" => bollard::models::RestartPolicyNameEnum::ALWAYS,
                    "unless-stopped" => bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED,
                    "on-failure" => bollard::models::RestartPolicyNameEnum::ON_FAILURE,
                    _ => bollard::models::RestartPolicyNameEnum::NO,
                }),
                maximum_retry_count: None,
            }
        });

        let client = self.client.clone();

        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                let host_config = HostConfig {
                    network_mode: network_mode_owned.clone(),
                    privileged: Some(privileged),
                    mounts: if docker_mounts.is_empty() { None } else { Some(docker_mounts) },
                    port_bindings: if port_bindings.is_empty() { None } else { Some(port_bindings) },
                    restart_policy: restart,
                    ..Default::default()
                };

                let config = ContainerCreateConfig {
                    image: Some(image_owned.clone()),
                    hostname: hostname_owned,
                    working_dir: workdir_owned,
                    cmd: cmd,
                    env: if env_vec.is_empty() { None } else { Some(env_vec) },
                    labels: if labels_owned.is_empty() { None } else { Some(labels_owned) },
                    exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
                    host_config: Some(host_config),
                    ..Default::default()
                };

                let opts = CreateContainerOptions { name: name_owned.as_str(), platform: None };

                let response = client
                    .create_container(Some(opts), config)
                    .await
                    .map_err(|e| errors::CommandFailed::new_err(format!(
                        "Failed to create container {name_owned}: {e}"
                    )))?;

                dim_log!("[Container] Created: {name_owned} ({})", &response.id[..12]);
                Ok(response.id)
            })
        })
    }

    /// Start a container by name.
    #[pyo3(signature = (name))]
    fn start(&self, py: Python<'_>, name: &str) -> PyResult<()> {
        dim_log!("[Container] Starting: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
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
        dim_log!("[Container] Stopping: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                client
                    .stop_container(&name, Some(StopContainerOptions { t: timeout }))
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to stop {name}: {e}")))?;
                Ok(())
            })
        })
    }

    /// Remove a container.
    #[pyo3(signature = (name, force = false))]
    fn remove(&self, py: Python<'_>, name: &str, force: bool) -> PyResult<()> {
        dim_log!("[Container] Removing: {name}");
        let client = self.client.clone();
        let name = name.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
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
        dim_log!("[Container Exec] {name} :: {cmd}");
        let client = self.client.clone();
        let name = name.to_string();
        let cmd = cmd.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
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

                if let StartExecResults::Attached { mut output, .. } = client
                    .start_exec(&exec.id, None)
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to start exec: {e}")))?
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
        dim_log!("[Container] Pulling: {image}:{tag}");
        let client = self.client.clone();
        let image = image.to_string();
        let tag = tag.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
                use bollard::image::CreateImageOptions;
                let opts = CreateImageOptions {
                    from_image: image.as_str(),
                    tag: tag.as_str(),
                    ..Default::default()
                };
                let mut stream = client.create_image(Some(opts), None, None);
                while let Some(result) = stream.next().await {
                    result.map_err(|e| PyRuntimeError::new_err(format!("Pull failed: {e}")))?;
                }
                dim_log!("[Container] Pulled: {image}:{tag}");
                Ok(())
            })
        })
    }

    /// Check if a container exists by name.
    #[pyo3(signature = (name))]
    fn exists(&self, py: Python<'_>, name: &str) -> PyResult<bool> {
        let client = self.client.clone();
        let name = name.to_string();
        let rt = crate::runtime::get()?;
        py.allow_threads(|| {
            rt.block_on(async {
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
    let rt = crate::runtime::get()?;
    py.allow_threads(|| {
        // Use 1-hour timeout to support long-running builds (npm install can take 15+ min)
        let client = if let Some(socket_path) = socket {
            Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION).map_err(
                |e| PyRuntimeError::new_err(format!("Failed to connect to {socket_path}: {e}")),
            )?
        } else {
            #[cfg(unix)]
            let c = Docker::connect_with_socket(
                "/var/run/docker.sock",
                120,
                bollard::API_DEFAULT_VERSION,
            ).map_err(|e| PyRuntimeError::new_err(format!("Failed to connect to Docker: {e}")))?;

            #[cfg(windows)]
            let c = Docker::connect_with_named_pipe(
                "//./pipe/docker_engine",
                120,
                bollard::API_DEFAULT_VERSION,
            ).map_err(|e| PyRuntimeError::new_err(format!("Failed to connect to Docker: {e}")))?;

            c
        };

        // Verify connection
        rt.block_on(async {
            client.ping().await.map_err(|e| {
                PyRuntimeError::new_err(format!(
                    "Docker/Podman daemon not reachable: {e}. Is Docker Desktop running?"
                ))
            })
        })?;

        dim_log!("[Container] Connected to Docker/Podman daemon");
        Ok(ContainerRuntime { client })
    })
}


// ── Helpers ────────────────────────────────────────────────────────────────

/// Create a tar.gz archive of a directory for Docker build context.
fn create_tar_archive(context_path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let buf = Vec::new();
    let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    // Walk the directory and add files
    for entry in walkdir::WalkDir::new(context_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative = path.strip_prefix(context_path).unwrap_or(path);

        if relative.as_os_str().is_empty() {
            continue;
        }

        if path.is_file() {
            archive.append_path_with_name(path, relative)?;
        } else if path.is_dir() && relative.as_os_str() != "" {
            archive.append_dir(relative, path)?;
        }
    }

    let encoder = archive.into_inner()?;
    encoder.finish()
}

/// Parse Python list of mount dicts into bollard Mount objects.
fn parse_mounts(mounts: Option<&Bound<'_, PyList>>) -> PyResult<Vec<Mount>> {
    let Some(mounts) = mounts else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();
    for item in mounts.iter() {
        let dict = item.downcast::<PyDict>()
            .map_err(|_| PyRuntimeError::new_err("Mount must be a dict"))?;

        let source: String = dict
            .get_item("source")?
            .ok_or_else(|| PyRuntimeError::new_err("Mount missing 'source'"))?
            .extract()?;
        let target: String = dict
            .get_item("target")?
            .ok_or_else(|| PyRuntimeError::new_err("Mount missing 'target'"))?
            .extract()?;
        let read_only: bool = dict
            .get_item("read_only")?
            .map(|v| v.extract().unwrap_or(false))
            .unwrap_or(false);

        result.push(Mount {
            target: Some(target),
            source: Some(source),
            typ: Some(MountTypeEnum::BIND),
            read_only: Some(read_only),
            ..Default::default()
        });
    }

    Ok(result)
}

/// Parse port mappings into exposed ports + host bindings.
fn parse_ports(
    ports: &HashMap<String, String>,
) -> (HashMap<String, HashMap<(), ()>>, HashMap<String, Option<Vec<PortBinding>>>) {
    let mut exposed = HashMap::new();
    let mut bindings = HashMap::new();

    for (container_port, host_port) in ports {
        exposed.insert(container_port.clone(), HashMap::new());
        bindings.insert(
            container_port.clone(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(host_port.clone()),
            }]),
        );
    }

    (exposed, bindings)
}
