//! File blocks — filesystem operations in Rust.

use std::fs;
use std::path::Path;

use pyo3::exceptions::{PyIOError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Copy a single file.
#[pyfunction]
#[pyo3(signature = (src, dest))]
pub fn copy(src: &str, dest: &str) -> PyResult<String> {
    eprintln!("[File Copy] {src} → {dest}");
    if let Some(parent) = Path::new(dest).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| PyIOError::new_err(format!("Failed to create {}: {e}", parent.display())))?;
    }
    fs::copy(src, dest)
        .map_err(|e| PyIOError::new_err(format!("Failed to copy {src} → {dest}: {e}")))?;
    Ok(dest.to_string())
}

/// Copy a directory recursively.
#[pyfunction]
#[pyo3(signature = (src, dest, clean = false))]
pub fn copy_tree(src: &str, dest: &str, clean: bool) -> PyResult<String> {
    eprintln!("[File Copy Tree] {src} → {dest}{}", if clean { " (clean)" } else { "" });
    let dest_path = Path::new(dest);
    if clean && dest_path.exists() {
        fs::remove_dir_all(dest_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to clean {dest}: {e}")))?;
    }
    copy_dir_recursive(Path::new(src), dest_path)?;
    Ok(dest.to_string())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> PyResult<()> {
    fs::create_dir_all(dest)
        .map_err(|e| PyIOError::new_err(format!("Failed to create {}: {e}", dest.display())))?;
    for entry in fs::read_dir(src)
        .map_err(|e| PyIOError::new_err(format!("Failed to read {}: {e}", src.display())))?
    {
        let entry = entry.map_err(|e| PyIOError::new_err(format!("Dir entry error: {e}")))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| {
                PyIOError::new_err(format!(
                    "Failed to copy {} → {}: {e}",
                    src_path.display(),
                    dest_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

/// Read and parse a YAML file, returning a Python dict.
#[pyfunction]
#[pyo3(signature = (path))]
pub fn read_yaml(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    eprintln!("[File Read YAML] {path}");
    let content = fs::read_to_string(path)
        .map_err(|e| PyIOError::new_err(format!("Failed to read {path}: {e}")))?;
    let value: serde_json::Value = serde_yaml::from_str(&content)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse YAML {path}: {e}")))?;
    json_value_to_py(py, &value)
}

/// Write a Python dict to a YAML file.
#[pyfunction]
#[pyo3(signature = (path, data))]
pub fn write_yaml(path: &str, data: &Bound<'_, PyDict>) -> PyResult<String> {
    eprintln!("[File Write YAML] {path}");
    let json_str = pythonize_to_json(data)?;
    let value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PyRuntimeError::new_err(format!("JSON conversion error: {e}")))?;
    let yaml_str = serde_yaml::to_string(&value)
        .map_err(|e| PyRuntimeError::new_err(format!("YAML serialization error: {e}")))?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| PyIOError::new_err(format!("Failed to create dir: {e}")))?;
    }
    fs::write(path, &yaml_str)
        .map_err(|e| PyIOError::new_err(format!("Failed to write {path}: {e}")))?;
    Ok(path.to_string())
}

/// Read and parse a JSON file, returning a Python dict.
#[pyfunction]
#[pyo3(signature = (path))]
pub fn read_json(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    eprintln!("[File Read JSON] {path}");
    let content = fs::read_to_string(path)
        .map_err(|e| PyIOError::new_err(format!("Failed to read {path}: {e}")))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse JSON {path}: {e}")))?;
    json_value_to_py(py, &value)
}

/// Write a Python dict to a JSON file.
#[pyfunction]
#[pyo3(signature = (path, data))]
pub fn write_json(path: &str, data: &Bound<'_, PyDict>) -> PyResult<String> {
    eprintln!("[File Write JSON] {path}");
    let json_str = pythonize_to_json(data)?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| PyIOError::new_err(format!("Failed to create dir: {e}")))?;
    }
    // Pretty-print
    let value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| PyRuntimeError::new_err(format!("JSON conversion error: {e}")))?;
    let pretty = serde_json::to_string_pretty(&value)
        .map_err(|e| PyRuntimeError::new_err(format!("JSON serialization error: {e}")))?;
    fs::write(path, &pretty)
        .map_err(|e| PyIOError::new_err(format!("Failed to write {path}: {e}")))?;
    Ok(path.to_string())
}

/// Replace all occurrences of `old` with `new` in a file.
#[pyfunction]
#[pyo3(signature = (path, old, new))]
pub fn replace(path: &str, old: &str, new: &str) -> PyResult<String> {
    eprintln!(
        "[File Replace] {path} :: '{}'... → '{}'...",
        &old[..old.len().min(60)],
        &new[..new.len().min(60)]
    );
    let content = fs::read_to_string(path)
        .map_err(|e| PyIOError::new_err(format!("Failed to read {path}: {e}")))?;
    if !content.contains(old) {
        eprintln!("[File Replace] Pattern not found in {path}");
    }
    let updated = content.replace(old, new);
    fs::write(path, &updated)
        .map_err(|e| PyIOError::new_err(format!("Failed to write {path}: {e}")))?;
    Ok(path.to_string())
}

/// Check if a string/bytes value is empty or whitespace-only.
#[pyfunction]
#[pyo3(signature = (value))]
pub fn is_empty(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(s) => s.trim().is_empty(),
    }
}

// ── Helpers for Python ↔ JSON conversion ──

pub fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => Ok((*b).into_pyobject(py)?.to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().into_any().unbind())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.to_owned().into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_value_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

fn pythonize_to_json(data: &Bound<'_, PyDict>) -> PyResult<String> {
    pythonize_to_json_pub(data)
}

pub fn pythonize_to_json_pub(data: &Bound<'_, PyDict>) -> PyResult<String> {
    let json_mod = data.py().import("json")?;
    let result = json_mod.call_method1("dumps", (data,))?;
    result.extract::<String>()
}
