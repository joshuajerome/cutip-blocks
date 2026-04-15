//! Network blocks — Docker/Podman network management via bollard.

use std::collections::HashMap;

use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::container::ContainerRuntime;

/// Create a Docker/Podman network.
#[pyfunction]
#[pyo3(signature = (runtime, name, *, driver = "bridge", subnet = None, gateway = None))]
pub fn create(
    py: Python<'_>,
    runtime: &ContainerRuntime,
    name: &str,
    driver: &str,
    subnet: Option<&str>,
    gateway: Option<&str>,
) -> PyResult<()> {
    eprintln!("[Network] Creating: {name} (driver={driver})");
    let client = runtime.client().clone();
    let rt = crate::runtime::get()?;
    let name = name.to_string();
    let driver = driver.to_string();
    let subnet = subnet.map(|s| s.to_string());
    let gateway = gateway.map(|s| s.to_string());

    py.allow_threads(|| {
        rt.block_on(async {
            let mut ipam_config = HashMap::new();
            if let Some(ref s) = subnet {
                ipam_config.insert("Subnet".to_string(), s.clone());
            }
            if let Some(ref g) = gateway {
                ipam_config.insert("Gateway".to_string(), g.clone());
            }

            let ipam = if !ipam_config.is_empty() {
                Some(bollard::models::Ipam {
                    config: Some(vec![bollard::models::IpamConfig {
                        subnet: subnet.clone(),
                        gateway: gateway.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                })
            } else {
                None
            };

            let opts = CreateNetworkOptions {
                name: name.as_str(),
                driver: driver.as_str(),
                ipam: ipam.unwrap_or_default(),
                ..Default::default()
            };

            client.create_network(opts).await.map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to create network {name}: {e}"))
            })?;

            eprintln!("[Network] Created: {name}");
            Ok(())
        })
    })
}

/// Remove a Docker/Podman network.
#[pyfunction]
#[pyo3(signature = (runtime, name))]
pub fn remove(py: Python<'_>, runtime: &ContainerRuntime, name: &str) -> PyResult<()> {
    eprintln!("[Network] Removing: {name}");
    let client = runtime.client().clone();
    let rt = crate::runtime::get()?;
    let name = name.to_string();

    py.allow_threads(|| {
        rt.block_on(async {
            client.remove_network(&name).await.map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to remove network {name}: {e}"))
            })?;
            eprintln!("[Network] Removed: {name}");
            Ok(())
        })
    })
}

/// Check if a network exists.
#[pyfunction]
#[pyo3(signature = (runtime, name))]
pub fn exists(py: Python<'_>, runtime: &ContainerRuntime, name: &str) -> PyResult<bool> {
    let client = runtime.client().clone();
    let rt = crate::runtime::get()?;
    let name = name.to_string();

    py.allow_threads(|| {
        rt.block_on(async {
            match client
                .inspect_network(&name, None::<InspectNetworkOptions<String>>)
                .await
            {
                Ok(_) => Ok(true),
                Err(bollard::errors::Error::DockerResponseServerError {
                    status_code: 404, ..
                }) => Ok(false),
                Err(e) => Err(PyRuntimeError::new_err(format!(
                    "Failed to inspect network {name}: {e}"
                ))),
            }
        })
    })
}
