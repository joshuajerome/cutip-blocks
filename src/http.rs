//! HTTP blocks — GET, POST, PUT, DELETE via reqwest.
//!
//! Two surfaces:
//! - Module-level `get`/`post`/`put`/`delete` for one-shot requests.
//! - `HttpSession` for stateful flows that need a persistent cookie jar
//!   or non-default redirect handling (e.g. Keycloak's
//!   authorization-code + UPDATE_PASSWORD dance).

use std::collections::HashMap;
use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use reqwest::cookie::Jar;
use reqwest::redirect;

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
    /// Response headers, lowercase keys. Multi-value headers are
    /// comma-joined (HTTP/1.1 §4.2 list semantics). `set-cookie` is
    /// special-cased — see `set_cookies`.
    pub headers_map: HashMap<String, String>,
    /// All Set-Cookie header values, in order. Kept separate from
    /// `headers_map` because Set-Cookie is the one header that doesn't
    /// follow list semantics and must not be folded together.
    pub set_cookies: Vec<String>,
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

    /// Response headers as a dict (lowercase keys, comma-joined values).
    /// Use `set_cookies` for Set-Cookie (which violates list semantics).
    #[getter]
    fn headers(&self) -> HashMap<String, String> {
        self.headers_map.clone()
    }

    /// All Set-Cookie header values in order. Kept separate from
    /// `headers` because Set-Cookie cannot be safely comma-joined.
    #[getter]
    fn set_cookies(&self) -> Vec<String> {
        self.set_cookies.clone()
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
            let client = build_client(verify_tls, timeout_s, true, None)?;
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

/// Send an HTTP POST request.
///
/// One of `json`, `body`, or `form` may be supplied (mutually exclusive).
/// `form` sends `application/x-www-form-urlencoded`; `json` sends
/// `application/json`; `body` sends a raw string with no Content-Type
/// (caller can set one via `headers`).
#[pyfunction]
#[pyo3(signature = (url, *, json = None, body = None, form = None, headers = None, verify_tls = true, timeout_s = 30))]
pub fn post(
    py: Python<'_>,
    url: &str,
    json: Option<&Bound<'_, PyDict>>,
    body: Option<&str>,
    form: Option<HashMap<String, String>>,
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
            let client = build_client(verify_tls, timeout_s, true, None)?;
            let req = client.post(&url);
            let req = apply_headers(req, &headers);
            let req = apply_body(req, json_body.as_deref(), str_body.as_deref(), form.as_ref());
            let resp = req
                .send()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("HTTP POST failed: {e}")))?;
            to_response(resp).await
        })
    })
}

/// Send an HTTP PUT request. Same body semantics as `post`.
#[pyfunction]
#[pyo3(signature = (url, *, json = None, body = None, form = None, headers = None, verify_tls = true, timeout_s = 30))]
pub fn put(
    py: Python<'_>,
    url: &str,
    json: Option<&Bound<'_, PyDict>>,
    body: Option<&str>,
    form: Option<HashMap<String, String>>,
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
            let client = build_client(verify_tls, timeout_s, true, None)?;
            let req = client.put(&url);
            let req = apply_headers(req, &headers);
            let req = apply_body(req, json_body.as_deref(), str_body.as_deref(), form.as_ref());
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
            let client = build_client(verify_tls, timeout_s, true, None)?;
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

/// Persistent HTTP session. Created via `http_session()`.
///
/// Holds a single `reqwest::Client` that:
///   - persists cookies across requests (essential for OAuth2 / Keycloak)
///   - uses a configurable redirect policy (`follow_redirects=False` to
///     read 302 `Location` headers manually)
///   - applies default headers to every request
#[pyclass]
pub struct HttpSession {
    inner: Option<HttpSessionInner>,
}

struct HttpSessionInner {
    client: reqwest::Client,
    default_headers: HashMap<String, String>,
}

#[pymethods]
impl HttpSession {
    #[pyo3(signature = (url, *, headers = None))]
    fn get(
        &self,
        py: Python<'_>,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> PyResult<HttpResponse> {
        let inner = self.require()?;
        let url_owned = url.to_string();
        dim_log!("[HTTP GET] {url}");
        let merged = merge_headers(&inner.default_headers, headers.as_ref());
        let client = inner.client.clone();
        py.allow_threads(|| {
            let rt = get_runtime()?;
            rt.block_on(async {
                let req = client.get(&url_owned);
                let req = apply_headers(req, &merged);
                let resp = req
                    .send()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("HTTP GET failed: {e}")))?;
                to_response(resp).await
            })
        })
    }

    #[pyo3(signature = (url, *, json = None, body = None, form = None, headers = None))]
    fn post(
        &self,
        py: Python<'_>,
        url: &str,
        json: Option<&Bound<'_, PyDict>>,
        body: Option<&str>,
        form: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
    ) -> PyResult<HttpResponse> {
        let inner = self.require()?;
        let url_owned = url.to_string();
        dim_log!("[HTTP POST] {url}");
        let merged = merge_headers(&inner.default_headers, headers.as_ref());
        let json_body = match json {
            Some(d) => Some(crate::file::pythonize_to_json_pub(d)?),
            None => None,
        };
        let str_body = body.map(|s| s.to_string());
        let client = inner.client.clone();
        py.allow_threads(|| {
            let rt = get_runtime()?;
            rt.block_on(async {
                let req = client.post(&url_owned);
                let req = apply_headers(req, &merged);
                let req = apply_body(req, json_body.as_deref(), str_body.as_deref(), form.as_ref());
                let resp = req
                    .send()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("HTTP POST failed: {e}")))?;
                to_response(resp).await
            })
        })
    }

    #[pyo3(signature = (url, *, json = None, body = None, form = None, headers = None))]
    fn put(
        &self,
        py: Python<'_>,
        url: &str,
        json: Option<&Bound<'_, PyDict>>,
        body: Option<&str>,
        form: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
    ) -> PyResult<HttpResponse> {
        let inner = self.require()?;
        let url_owned = url.to_string();
        dim_log!("[HTTP PUT] {url}");
        let merged = merge_headers(&inner.default_headers, headers.as_ref());
        let json_body = match json {
            Some(d) => Some(crate::file::pythonize_to_json_pub(d)?),
            None => None,
        };
        let str_body = body.map(|s| s.to_string());
        let client = inner.client.clone();
        py.allow_threads(|| {
            let rt = get_runtime()?;
            rt.block_on(async {
                let req = client.put(&url_owned);
                let req = apply_headers(req, &merged);
                let req = apply_body(req, json_body.as_deref(), str_body.as_deref(), form.as_ref());
                let resp = req
                    .send()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("HTTP PUT failed: {e}")))?;
                to_response(resp).await
            })
        })
    }

    #[pyo3(signature = (url, *, headers = None))]
    fn delete(
        &self,
        py: Python<'_>,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> PyResult<HttpResponse> {
        let inner = self.require()?;
        let url_owned = url.to_string();
        dim_log!("[HTTP DELETE] {url}");
        let merged = merge_headers(&inner.default_headers, headers.as_ref());
        let client = inner.client.clone();
        py.allow_threads(|| {
            let rt = get_runtime()?;
            rt.block_on(async {
                let req = client.delete(&url_owned);
                let req = apply_headers(req, &merged);
                let resp = req
                    .send()
                    .await
                    .map_err(|e| PyRuntimeError::new_err(format!("HTTP DELETE failed: {e}")))?;
                to_response(resp).await
            })
        })
    }

    /// Drop the underlying client and cookie jar.
    fn close(&mut self) -> PyResult<()> {
        if self.inner.take().is_some() {
            dim_log!("[HTTP] Session closed");
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        self.close()
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            Some(_) => "HttpSession(open)".to_string(),
            None => "HttpSession(closed)".to_string(),
        }
    }
}

impl HttpSession {
    fn require(&self) -> PyResult<&HttpSessionInner> {
        self.inner
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("HTTP session is closed"))
    }
}

/// Open a persistent HTTP session.
///
/// - `verify_tls=False` accepts self-signed certs (SFM/dev VMs).
/// - `follow_redirects=False` returns the 3xx response directly so the
///   caller can read the `Location` header (needed for OAuth2 flows).
/// - `headers` are sent on every request unless overridden per-call.
#[pyfunction]
#[pyo3(name = "http_session", signature = (*, verify_tls = true, follow_redirects = true, timeout_s = 30, headers = None))]
pub fn http_session(
    verify_tls: bool,
    follow_redirects: bool,
    timeout_s: u64,
    headers: Option<HashMap<String, String>>,
) -> PyResult<HttpSession> {
    let client = build_client(verify_tls, timeout_s, follow_redirects, Some(Arc::new(Jar::default())))?;
    dim_log!(
        "[HTTP] Session opened (verify_tls={}, follow_redirects={}, timeout={}s)",
        verify_tls,
        follow_redirects,
        timeout_s
    );
    Ok(HttpSession {
        inner: Some(HttpSessionInner {
            client,
            default_headers: headers.unwrap_or_default(),
        }),
    })
}

fn build_client(
    verify_tls: bool,
    timeout_s: u64,
    follow_redirects: bool,
    cookie_jar: Option<Arc<Jar>>,
) -> PyResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .danger_accept_invalid_certs(!verify_tls)
        .timeout(std::time::Duration::from_secs(timeout_s));
    if follow_redirects {
        // reqwest's default is `limited(10)`, but be explicit.
        builder = builder.redirect(redirect::Policy::limited(10));
    } else {
        builder = builder.redirect(redirect::Policy::none());
    }
    if let Some(jar) = cookie_jar {
        builder = builder.cookie_provider(jar);
    } else {
        builder = builder.cookie_store(true);
    }
    builder
        .build()
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to build HTTP client: {e}")))
}

fn apply_headers(
    mut req: reqwest::RequestBuilder,
    headers: &HashMap<String, String>,
) -> reqwest::RequestBuilder {
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }
    req
}

fn apply_body(
    req: reqwest::RequestBuilder,
    json_body: Option<&str>,
    str_body: Option<&str>,
    form: Option<&HashMap<String, String>>,
) -> reqwest::RequestBuilder {
    if let Some(j) = json_body {
        req.header("Content-Type", "application/json").body(j.to_string())
    } else if let Some(f) = form {
        // reqwest sets Content-Type: application/x-www-form-urlencoded.
        req.form(f)
    } else if let Some(b) = str_body {
        req.body(b.to_string())
    } else {
        req
    }
}

fn merge_headers(
    defaults: &HashMap<String, String>,
    overrides: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut out = defaults.clone();
    if let Some(o) = overrides {
        for (k, v) in o {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

async fn to_response(resp: reqwest::Response) -> PyResult<HttpResponse> {
    let status = resp.status().as_u16();
    let ok = resp.status().is_success();

    // Capture headers before consuming the body. Set-Cookie kept separate
    // (cannot be safely comma-joined per RFC 6265).
    let mut headers_map: HashMap<String, String> = HashMap::new();
    let mut set_cookies: Vec<String> = Vec::new();
    for (name, value) in resp.headers().iter() {
        let lname = name.as_str().to_ascii_lowercase();
        let v = value.to_str().unwrap_or("").to_string();
        if lname == "set-cookie" {
            set_cookies.push(v);
            continue;
        }
        headers_map
            .entry(lname)
            .and_modify(|existing| {
                existing.push_str(", ");
                existing.push_str(&v);
            })
            .or_insert(v);
    }

    let raw = resp
        .bytes()
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read response: {e}")))?
        .to_vec();
    let text = String::from_utf8_lossy(&raw).into_owned();
    Ok(HttpResponse {
        status_code: status,
        text,
        ok,
        raw,
        headers_map,
        set_cookies,
    })
}
