//! Service blocks — poll for readiness, wait for exit.

use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

/// Poll an HTTP endpoint until it returns a success status code.
#[pyfunction]
#[pyo3(signature = (url, *, retries = 30, interval_s = 1, verify_tls = true))]
pub fn poll_until_ready(
    py: Python<'_>,
    url: &str,
    retries: u32,
    interval_s: u64,
    verify_tls: bool,
) -> PyResult<()> {
    eprintln!("[Service] Polling {url} (max {retries} attempts, {interval_s}s interval)");
    let url = url.to_string();

    py.allow_threads(|| {
        let rt =
            Runtime::new().map_err(|e| PyRuntimeError::new_err(format!("Runtime error: {e}")))?;

        rt.block_on(async {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(!verify_tls)
                .timeout(Duration::from_secs(5))
                .build()
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP client error: {e}")))?;

            for attempt in 1..=retries {
                match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        eprintln!("[Service] Ready after {attempt} attempt(s): {url}");
                        return Ok(());
                    }
                    Ok(resp) => {
                        eprintln!(
                            "[Service] Attempt {attempt}/{retries}: HTTP {}",
                            resp.status().as_u16()
                        );
                    }
                    Err(e) => {
                        eprintln!("[Service] Attempt {attempt}/{retries}: {e}");
                    }
                }
                if attempt < retries {
                    tokio::time::sleep(Duration::from_secs(interval_s)).await;
                }
            }

            Err(PyRuntimeError::new_err(format!(
                "Service not ready after {retries} attempts: {url}"
            )))
        })
    })
}

/// Wait for a container to exit (via Docker API).
#[pyfunction]
#[pyo3(signature = (runtime, container_name, *, timeout_s = 120))]
pub fn wait_for_exit(
    py: Python<'_>,
    runtime: &crate::container::ContainerRuntime,
    container_name: &str,
    timeout_s: u64,
) -> PyResult<i64> {
    eprintln!("[Service] Waiting for container {container_name} to exit (timeout {timeout_s}s)");
    let client = runtime.client().clone();
    let rt = runtime.runtime();
    let name = container_name.to_string();

    py.allow_threads(|| {
        rt.block_on(async {
            use bollard::container::WaitContainerOptions;
            use futures_util::StreamExt;

            let timeout = tokio::time::timeout(Duration::from_secs(timeout_s), async {
                let mut stream = client.wait_container(
                    &name,
                    Some(WaitContainerOptions {
                        condition: "not-running",
                    }),
                );
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(exit) => {
                            let code = exit.status_code;
                            eprintln!("[Service] Container {name} exited with code {code}");
                            return Ok(code);
                        }
                        Err(e) => {
                            return Err(PyRuntimeError::new_err(format!(
                                "Wait error for {name}: {e}"
                            )));
                        }
                    }
                }
                Err(PyRuntimeError::new_err(format!(
                    "Wait stream ended without exit for {name}"
                )))
            })
            .await;

            match timeout {
                Ok(result) => result,
                Err(_) => Err(PyRuntimeError::new_err(format!(
                    "Timeout waiting for {name} to exit after {timeout_s}s"
                ))),
            }
        })
    })
}
