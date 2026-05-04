//! JS module blocks — parse / render simple JavaScript object-literal modules.
//!
//! This is **not** a JS engine. It handles the common config-file shape:
//!
//! ```js
//! const PROXY_CONFIG = {
//!   '/redfish': { target: 'http://x', secure: false, /* ... */ },
//!   // ...
//! };
//!
//! module.exports = PROXY_CONFIG;
//! ```
//!
//! and the equally-common direct form:
//!
//! ```js
//! module.exports = { /* ... */ };
//! ```
//!
//! The body is parsed as JSON5 (single quotes, unquoted keys, trailing
//! commas, comments — all OK). Anything dynamic (env vars, imports,
//! function values, computed keys) will fail to parse — that's a
//! deliberate boundary.
//!
//! The writer preserves the input wrapper style when possible: if the
//! file used `const NAME = ...; module.exports = NAME;`, the rewrite
//! keeps the same keyword + identifier. Otherwise it emits the direct
//! `module.exports = ...;` form.

use std::fs;
use std::path::Path;

use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::dim_log;
use crate::file::{json_value_to_py, pythonize_to_json_pub};

/// Detected wrapper around the object literal.
#[derive(Debug, Clone)]
enum Wrapper {
    /// `module.exports = {...};`
    Direct,
    /// `const|let|var IDENT = {...};\nmodule.exports = IDENT;`
    Named { keyword: String, ident: String },
}

/// Parse a JS module file body and return the wrapper style + raw object literal text.
fn split_wrapper(content: &str) -> Result<(Wrapper, String), String> {
    // Find module.exports = ... ; (must exist)
    let me_idx = content
        .find("module.exports")
        .ok_or("file does not contain `module.exports`")?;
    let after_me = &content[me_idx + "module.exports".len()..];
    let eq_idx = after_me
        .find('=')
        .ok_or("expected `=` after module.exports")?;
    let rhs = after_me[eq_idx + 1..].trim_start();

    // Two cases: rhs starts with `{` (direct) or with an identifier (named).
    if rhs.starts_with('{') {
        let body = extract_balanced(rhs, '{', '}')
            .ok_or("unbalanced braces in module.exports object literal")?;
        return Ok((Wrapper::Direct, body.to_string()));
    }

    // Named form: rhs is an identifier; trace back to its const|let|var decl.
    let ident: String = rhs
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if ident.is_empty() {
        return Err(format!("could not parse identifier from `{rhs}`"));
    }

    // Search above me_idx for `const|let|var IDENT = {`
    let preamble = &content[..me_idx];
    for keyword in ["const", "let", "var"] {
        let pat_start = format!("{keyword} {ident}");
        if let Some(decl_idx) = preamble.find(&pat_start) {
            let after_decl = &preamble[decl_idx + pat_start.len()..];
            let eq_pos = after_decl.find('=').ok_or(format!(
                "expected `=` after `{keyword} {ident}` declaration"
            ))?;
            let object_start = after_decl[eq_pos + 1..].trim_start();
            let body = extract_balanced(object_start, '{', '}')
                .ok_or("unbalanced braces in named declaration object literal")?;
            return Ok((
                Wrapper::Named {
                    keyword: keyword.to_string(),
                    ident: ident.clone(),
                },
                body.to_string(),
            ));
        }
    }

    Err(format!(
        "module.exports = {ident}; but no const|let|var {ident} = ... declaration found above"
    ))
}

/// Extract a balanced `{...}` (or `[...]`) substring starting at the first
/// occurrence of `open` in `s`. Tracks depth, ignores braces inside string
/// literals (single, double) and line/block comments.
fn extract_balanced(s: &str, open: char, close: char) -> Option<&str> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(|&b| b == open as u8)?;
    let mut depth = 0u32;
    let mut i = start;
    let mut in_str: Option<u8> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < bytes.len() {
        let c = bytes[i];

        if in_line_comment {
            if c == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && bytes.get(i + 1) == Some(&b'/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }

        // Not in a string / comment.
        match c {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                in_line_comment = true;
                i += 2;
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                in_block_comment = true;
                i += 2;
                continue;
            }
            b'"' | b'\'' => {
                in_str = Some(c);
                i += 1;
                continue;
            }
            _ => {}
        }

        if c == open as u8 {
            depth += 1;
        } else if c == close as u8 {
            depth -= 1;
            if depth == 0 {
                return Some(&s[start..=i]);
            }
        }
        i += 1;
    }
    None
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Read a JS module file and return the exported object as a Python dict.
///
/// Supports `const|let|var IDENT = {...}; module.exports = IDENT;` and
/// `module.exports = {...};` shapes. Body is parsed as JSON5.
#[pyfunction]
#[pyo3(signature = (path))]
pub fn js_read_module(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    dim_log!("[JS Read Module] {path}");
    let content = fs::read_to_string(path)
        .map_err(|e| PyIOError::new_err(format!("Failed to read {path}: {e}")))?;
    let (_wrapper, body) = split_wrapper(&content)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse {path}: {e}")))?;
    let value: serde_json::Value = json5::from_str(&body).map_err(|e| {
        PyRuntimeError::new_err(format!("Failed to parse JSON5 body of {path}: {e}"))
    })?;
    json_value_to_py(py, &value)
}

/// Write a Python dict back as a JS module, preserving the wrapper style of
/// the existing file (or defaulting to `module.exports = ...;` if the file
/// is new).
#[pyfunction]
#[pyo3(signature = (path, data))]
pub fn js_write_module(path: &str, data: &Bound<'_, PyDict>) -> PyResult<String> {
    dim_log!("[JS Write Module] {path}");

    // Detect existing wrapper, if any.
    let wrapper = match fs::read_to_string(path) {
        Ok(existing) => match split_wrapper(&existing) {
            Ok((w, _)) => w,
            Err(_) => Wrapper::Direct,
        },
        Err(_) => Wrapper::Direct,
    };

    // Convert dict → JSON5 body (we serialize as JSON; JSON is a strict
    // subset of JSON5, so the output round-trips correctly).
    let json_str = pythonize_to_json_pub(data)?;
    let value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        PyRuntimeError::new_err(format!("Internal: failed to round-trip dict to JSON: {e}"))
    })?;
    let pretty_body = serde_json::to_string_pretty(&value)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize body: {e}")))?;

    let rendered = match wrapper {
        Wrapper::Direct => format!("module.exports = {pretty_body};\n"),
        Wrapper::Named { keyword, ident } => {
            format!("{keyword} {ident} = {pretty_body};\n\nmodule.exports = {ident};\n")
        }
    };

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| PyIOError::new_err(format!("Failed to create dir: {e}")))?;
    }
    fs::write(path, &rendered)
        .map_err(|e| PyIOError::new_err(format!("Failed to write {path}: {e}")))?;
    Ok(path.to_string())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_wrapper_direct() {
        let src = "module.exports = { a: 1 };";
        let (w, body) = split_wrapper(src).unwrap();
        assert!(matches!(w, Wrapper::Direct));
        assert_eq!(body, "{ a: 1 }");
    }

    #[test]
    fn split_wrapper_named() {
        let src = "const PROXY_CONFIG = {\n  '/x': { target: 'http://h' },\n};\n\nmodule.exports = PROXY_CONFIG;";
        let (w, body) = split_wrapper(src).unwrap();
        match w {
            Wrapper::Named { keyword, ident } => {
                assert_eq!(keyword, "const");
                assert_eq!(ident, "PROXY_CONFIG");
            }
            _ => panic!("expected Named wrapper"),
        }
        assert!(body.starts_with("{"));
        assert!(body.contains("/x"));
        assert!(body.contains("target: 'http://h'"));
    }

    #[test]
    fn split_wrapper_let_keyword() {
        let src = "let X = { a: 1 };\nmodule.exports = X;";
        let (w, _body) = split_wrapper(src).unwrap();
        match w {
            Wrapper::Named { keyword, .. } => assert_eq!(keyword, "let"),
            _ => panic!(),
        }
    }

    #[test]
    fn split_wrapper_no_module_exports_errors() {
        let result = split_wrapper("const X = { a: 1 };");
        assert!(result.is_err());
    }

    #[test]
    fn extract_balanced_simple() {
        assert_eq!(
            extract_balanced("hi { a: 1 } there", '{', '}'),
            Some("{ a: 1 }")
        );
    }

    #[test]
    fn extract_balanced_nested() {
        assert_eq!(
            extract_balanced("{ a: { b: 1 }, c: 2 }", '{', '}'),
            Some("{ a: { b: 1 }, c: 2 }")
        );
    }

    #[test]
    fn extract_balanced_string_with_close_brace() {
        // `}` inside a string shouldn't end the object
        assert_eq!(
            extract_balanced("{ a: 'has } in string' }", '{', '}'),
            Some("{ a: 'has } in string' }")
        );
    }

    #[test]
    fn extract_balanced_line_comment() {
        let s = "{ a: 1, // closer would-be: }\n  b: 2 }";
        assert_eq!(extract_balanced(s, '{', '}'), Some(s));
    }

    #[test]
    fn extract_balanced_block_comment() {
        let s = "{ a: 1 /* fake } here */, b: 2 }";
        assert_eq!(extract_balanced(s, '{', '}'), Some(s));
    }

    #[test]
    fn parses_real_proxy_conf() {
        let src = r#"const PROXY_CONFIG = {
  '/redfish': {
    target: 'http://100.94.115.88',
    secure: false,
    logLevel: 'debug',
    auth: 'sfmadmin:Dellsfm@force10',
  },
  '/api': {
    target: 'http://100.94.115.88',
    secure: false,
    logLevel: 'debug',
    auth: 'sfmadmin:Dellsfm@force10',
  },
};

module.exports = PROXY_CONFIG;
"#;
        let (_, body) = split_wrapper(src).unwrap();
        let value: serde_json::Value = json5::from_str(&body).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(
            obj["/redfish"]["target"].as_str().unwrap(),
            "http://100.94.115.88"
        );
        assert_eq!(obj["/api"]["secure"].as_bool().unwrap(), false);
    }
}
