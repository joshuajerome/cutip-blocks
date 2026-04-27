//! HTTP blocks — GET, POST, PUT, DELETE via reqwest.

use std::collections::HashMap;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Result of an HTTP request.
#[pyclass]
#[derive(Clone)]
pub struct HttpResponse {
    #[pyo3(get)]
    pub status_code: u16,
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub ok: bool,
    /// Raw response body bytes. Use this for binary downloads (tarballs,
    /// images, anything not text). `text` is a UTF-8 decode of these bytes,
    /// lossy for non-UTF-8 binary content.
    pub raw: Vec<u8>,
}

#[pymethods]
impl HttpResponse {
    /// Parse the response body as JSON, returning a Python dict.
    fn json(&self, py: Python<'_>) -> PyResult<PyObject> {
        let value: serde_json::Value = serde_json::from_str(&self.text)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON parse error: {e}")))?;
        crate::file::json_value_to_py(py, &value)
    }

    /// Raw response body as Python `bytes`. Use this for binary downloads.
    #[getter]
    fn bytes<'py>(&self, py: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        pyo3::types::PyBytes::new(py, &self.raw)
    }

    fn __repr__(&self) -> String {
        format!(
            "HttpResponse(status={}, ok={}, len={})",
            self.status_code,
            self.ok,
            self.raw.len()
        )
    }
}

fn get_runtime() -> PyResult<&'static tokio::runtime::Runtime> {
    crate::runtime::get()
}

/// Send an HTTP GET request.
#[pyfunction]
#[pyo3(signature = (url, *, headers = None, verify_tls = true, timeout_s = 30))]
pub fn get(
    py: Python<'_>,
    url: &str,
    headers: Option<HashMap<String, String>>,
    verify_tls: bool,
    timeout_s: u64,
) -> PyResult<HttpResponse> {
    dim_log!("[HTTP GET] {url}");
    let url = url.to_string();
    let headers = headers.unwrap_or_default();

    py.allow_threads(|| {
        let rt = get_runtime()?;
        rt.block_on(async {
            let client = build_client(verify_tls, timeout_s)?;
            let mut req = client.get(&url);
            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP GET failed: {e}")))?;
            to_response(resp).await
        })
    })
}

/// Send an HTTP POST request with JSON body.
#[pyfunction]
#[pyo3(signature = (url, *, json = None, body = None, headers = None, verify_tls = true, timeout_s = 30))]
pub fn post(
    py: Python<'_>,
    url: &str,
    json: Option<&Bound<'_, PyDict>>,
    body: Option<&str>,
    headers: Option<HashMap<String, String>>,
    verify_tls: bool,
    timeout_s: u64,
) -> PyResult<HttpResponse> {
    dim_log!("[HTTP POST] {url}");
    let url = url.to_string();
    let headers = headers.unwrap_or_default();
    let json_body = match json {
        Some(d) => Some(crate::file::pythonize_to_json_pub(d)?),
        None => None,
    };
    let str_body = body.map(|s| s.to_string());

    py.allow_threads(|| {
        let rt = get_runtime()?;
        rt.block_on(async {
            let client = build_client(verify_tls, timeout_s)?;
            let mut req = client.post(&url);
            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }
            if let Some(j) = &json_body {
                req = req
                    .header("Content-Type", "application/json")
                    .body(j.clone());
            } else if let Some(b) = &str_body {
                req = req.body(b.clone());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP POST failed: {e}")))?;
            to_response(resp).await
        })
    })
}

/// Send an HTTP PUT request with JSON body.
#[pyfunction]
#[pyo3(signature = (url, *, json = None, body = None, headers = None, verify_tls = true, timeout_s = 30))]
pub fn put(
    py: Python<'_>,
    url: &str,
    json: Option<&Bound<'_, PyDict>>,
    body: Option<&str>,
    headers: Option<HashMap<String, String>>,
    verify_tls: bool,
    timeout_s: u64,
) -> PyResult<HttpResponse> {
    dim_log!("[HTTP PUT] {url}");
    let url = url.to_string();
    let headers = headers.unwrap_or_default();
    let json_body = match json {
        Some(d) => Some(crate::file::pythonize_to_json_pub(d)?),
        None => None,
    };
    let str_body = body.map(|s| s.to_string());

    py.allow_threads(|| {
        let rt = get_runtime()?;
        rt.block_on(async {
            let client = build_client(verify_tls, timeout_s)?;
            let mut req = client.put(&url);
            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }
            if let Some(j) = &json_body {
                req = req
                    .header("Content-Type", "application/json")
                    .body(j.clone());
            } else if let Some(b) = &str_body {
                req = req.body(b.clone());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP PUT failed: {e}")))?;
            to_response(resp).await
        })
    })
}

/// Send an HTTP DELETE request.
#[pyfunction]
#[pyo3(signature = (url, *, headers = None, verify_tls = true, timeout_s = 30))]
pub fn delete(
    py: Python<'_>,
    url: &str,
    headers: Option<HashMap<String, String>>,
    verify_tls: bool,
    timeout_s: u64,
) -> PyResult<HttpResponse> {
    dim_log!("[HTTP DELETE] {url}");
    let url = url.to_string();
    let headers = headers.unwrap_or_default();

    py.allow_threads(|| {
        let rt = get_runtime()?;
        rt.block_on(async {
            let client = build_client(verify_tls, timeout_s)?;
            let mut req = client.delete(&url);
            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP DELETE failed: {e}")))?;
            to_response(resp).await
        })
    })
}

fn build_client(verify_tls: bool, timeout_s: u64) -> PyResult<reqwest::Client> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(!verify_tls)
        .timeout(std::time::Duration::from_secs(timeout_s))
        .build()
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to build HTTP client: {e}")))
}

async fn to_response(resp: reqwest::Response) -> PyResult<HttpResponse> {
    let status = resp.status().as_u16();
    let ok = resp.status().is_success();
    let raw = resp
        .bytes()
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read response: {e}")))?
        .to_vec();
    // text is a lossy UTF-8 decode of raw — fine for text responses, expected
    // to contain replacement chars for binary content (caller should use .bytes).
    let text = String::from_utf8_lossy(&raw).into_owned();
    Ok(HttpResponse {
        status_code: status,
        text,
        ok,
        raw,
    })
}
