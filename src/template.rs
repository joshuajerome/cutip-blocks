//! Template rendering blocks — variable substitution in files and strings.
//!
//! Renders `{{ var }}` placeholders from a variables dict.
//! Similar to Jinja2 but intentionally simpler — no filters, no loops, no conditionals.
//! For complex templating, use the `config` module's `render_template` / `substitute_vars`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

use crate::errors;

/// Render a template file, replacing {{ key }} placeholders with values.
///
/// Reads the template file, substitutes variables, writes the result to dest.
/// If dest is not specified, returns the rendered string without writing.
#[pyfunction]
#[pyo3(signature = (src, vars, dest = None))]
pub fn render(
    src: &str,
    vars: HashMap<String, String>,
    dest: Option<&str>,
) -> PyResult<String> {
    dim_log!("[Template] Rendering {src}");

    let template = fs::read_to_string(src)
        .map_err(|e| PyIOError::new_err(format!("Failed to read template {src}: {e}")))?;

    let rendered = substitute(&template, &vars);

    if let Some(dest_path) = dest {
        if let Some(parent) = Path::new(dest_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PyIOError::new_err(format!("Failed to create {}: {e}", parent.display())))?;
        }
        fs::write(dest_path, &rendered)
            .map_err(|e| PyIOError::new_err(format!("Failed to write {dest_path}: {e}")))?;
        dim_log!("[Template] Wrote {dest_path}");
    }

    Ok(rendered)
}

/// Render a template string (not a file), replacing {{ key }} placeholders.
#[pyfunction]
#[pyo3(signature = (text, vars))]
pub fn render_string(text: &str, vars: HashMap<String, String>) -> String {
    substitute(text, &vars)
}

/// Check that all {{ key }} placeholders in a template have values in vars.
/// Returns a list of missing keys.
#[pyfunction]
#[pyo3(signature = (src, vars))]
pub fn check(src: &str, vars: HashMap<String, String>) -> PyResult<Vec<String>> {
    let template = fs::read_to_string(src)
        .map_err(|e| PyIOError::new_err(format!("Failed to read template {src}: {e}")))?;

    let mut missing = Vec::new();
    let mut i = 0;
    let bytes = template.as_bytes();

    while i < bytes.len().saturating_sub(1) {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = template[i + 2..].find("}}") {
                let key = template[i + 2..i + 2 + end].trim();
                if !key.is_empty() && !vars.contains_key(key) {
                    missing.push(key.to_string());
                }
                i += end + 4;
                continue;
            }
        }
        i += 1;
    }

    missing.dedup();
    Ok(missing)
}

/// Replace {{ key }} placeholders in text with values from vars dict.
fn substitute(text: &str, vars: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in vars {
        // Match both {{ key }} and {{key}}
        let patterns = [format!("{{{{ {key} }}}}"), format!("{{{{{key}}}}}")];
        for pattern in &patterns {
            result = result.replace(pattern, value);
        }
    }
    result
}
