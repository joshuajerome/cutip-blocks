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
        let patterns = [format!("{{{{ {key} }}}}"), format!("{{{{{key}}}}}")];
        for pattern in &patterns {
            result = result.replace(pattern, value);
        }
    }
    result
}

/// Replace `{{ vars.key }}`, `{{ paths.key }}`, `{{ secrets.key }}`, and
/// `{{ globals.key.nested }}` placeholders in text.
///
/// Both spaced (`{{ ns.key }}`) and unspaced (`{{ns.key}}`) forms are
/// supported. Substitution order: vars → paths → secrets → globals.
///
/// `globals` keys are pre-flattened dotted paths (e.g. `"passwords.v22"`).
/// Use ``rsty.config.flatten`` (Python helper) to convert a nested dict
/// to that shape before passing.
#[pyfunction]
#[pyo3(signature = (text, vars, secrets = None, paths = None, globals = None))]
pub fn substitute_vars(
    text: &str,
    vars: HashMap<String, String>,
    secrets: Option<HashMap<String, String>>,
    paths: Option<HashMap<String, String>>,
    globals: Option<HashMap<String, String>>,
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
    if let Some(paths) = &paths {
        for (key, value) in paths {
            let patterns = [
                format!("{{{{ paths.{key} }}}}"),
                format!("{{{{paths.{key}}}}}"),
            ];
            for pattern in &patterns {
                result = result.replace(pattern, value);
            }
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
    if let Some(globals) = &globals {
        for (key, value) in globals {
            let patterns = [
                format!("{{{{ globals.{key} }}}}"),
                format!("{{{{globals.{key}}}}}"),
            ];
            for pattern in &patterns {
                result = result.replace(pattern, value);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn substitutes_paths_with_spaces() {
        let result = substitute_vars(
            "src: {{ paths.repo }}",
            map(&[]),
            None,
            Some(map(&[("repo", "/home/u/proj")])),
            None,
        );
        assert_eq!(result, "src: /home/u/proj");
    }

    #[test]
    fn substitutes_paths_without_spaces() {
        let result = substitute_vars(
            "src: {{paths.repo}}",
            map(&[]),
            None,
            Some(map(&[("repo", "/home/u/proj")])),
            None,
        );
        assert_eq!(result, "src: /home/u/proj");
    }

    #[test]
    fn substitutes_all_four_namespaces() {
        let result = substitute_vars(
            "{{ vars.a }}|{{ paths.b }}|{{ secrets.c }}|{{ globals.d.e }}",
            map(&[("a", "VA")]),
            Some(map(&[("c", "SC")])),
            Some(map(&[("b", "PB")])),
            Some(map(&[("d.e", "GD")])),
        );
        assert_eq!(result, "VA|PB|SC|GD");
    }

    #[test]
    fn paths_namespace_isolated_from_vars() {
        let result = substitute_vars(
            "v={{ vars.x }} p={{ paths.x }}",
            map(&[("x", "from-vars")]),
            None,
            Some(map(&[("x", "from-paths")])),
            None,
        );
        assert_eq!(result, "v=from-vars p=from-paths");
    }

    #[test]
    fn paths_omitted_leaves_placeholder() {
        let result = substitute_vars("src: {{ paths.repo }}", map(&[]), None, None, None);
        assert_eq!(result, "src: {{ paths.repo }}");
    }

    #[test]
    fn vars_only_call_unchanged() {
        let result = substitute_vars(
            "{{ vars.a }} {{ secrets.b }}",
            map(&[("a", "X")]),
            Some(map(&[("b", "Y")])),
            None,
            None,
        );
        assert_eq!(result, "X Y");
    }

    #[test]
    fn substitutes_globals_dotted() {
        let result = substitute_vars(
            "pw={{ globals.passwords.v22 }}",
            map(&[]),
            None,
            None,
            Some(map(&[("passwords.v22", "DefaultPw123")])),
        );
        assert_eq!(result, "pw=DefaultPw123");
    }

    #[test]
    fn globals_omitted_leaves_placeholder() {
        let result = substitute_vars("{{ globals.x.y }}", map(&[]), None, None, None);
        assert_eq!(result, "{{ globals.x.y }}");
    }
}
