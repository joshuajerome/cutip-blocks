//! Config blocks — template rendering and variable substitution.

use std::collections::HashMap;

use pyo3::prelude::*;

/// Replace `{{ key }}` placeholders in a template string with values from a vars dict.
#[pyfunction]
#[pyo3(signature = (template, vars))]
pub fn render_template(template: &str, vars: HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in &vars {
        // Match {{ key }} with optional whitespace inside braces
        let patterns = [
            format!("{{{{ {key} }}}}"),
            format!("{{{{{key}}}}}"),
        ];
        for pattern in &patterns {
            result = result.replace(pattern, value);
        }
    }
    result
}

/// Replace `{{ vars.key }}` and `{{ secrets.key }}` placeholders in text.
#[pyfunction]
#[pyo3(signature = (text, vars, secrets = None))]
pub fn substitute_vars(
    text: &str,
    vars: HashMap<String, String>,
    secrets: Option<HashMap<String, String>>,
) -> String {
    let mut result = text.to_string();
    for (key, value) in &vars {
        let patterns = [
            format!("{{{{ vars.{key} }}}}"),
            format!("{{{{vars.{key}}}}}"),
        ];
        for pattern in &patterns {
            result = result.replace(pattern, value);
        }
    }
    if let Some(secrets) = &secrets {
        for (key, value) in secrets {
            let patterns = [
                format!("{{{{ secrets.{key} }}}}"),
                format!("{{{{secrets.{key}}}}}"),
            ];
            for pattern in &patterns {
                result = result.replace(pattern, value);
            }
        }
    }
    result
}
