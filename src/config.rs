//! Config blocks — template rendering and variable substitution.

use std::collections::HashMap;

use pyo3::prelude::*;
use regex::Regex;

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
/// Whitespace-tolerant: `{{ns.key}}`, `{{ ns.key }}`, `{{ ns.key}}`,
/// `{{ns.key }}`, multi-space, and tabs all match. Substitution order:
/// vars → paths → secrets → globals.
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
    apply_namespace(&mut result, "vars", &vars);
    if let Some(paths) = &paths {
        apply_namespace(&mut result, "paths", paths);
    }
    if let Some(secrets) = &secrets {
        apply_namespace(&mut result, "secrets", secrets);
    }
    if let Some(globals) = &globals {
        apply_namespace(&mut result, "globals", globals);
    }
    result
}

/// Run regex substitution for one namespace. Pattern: `\{\{\s*ns\.key\s*\}\}`
/// — any (or no) whitespace inside the braces.
fn apply_namespace(result: &mut String, ns: &str, mapping: &HashMap<String, String>) {
    for (key, value) in mapping {
        let pattern = format!(
            r"\{{\{{\s*{}\.{}\s*\}}\}}",
            regex::escape(ns),
            regex::escape(key)
        );
        if let Ok(re) = Regex::new(&pattern) {
            *result = re.replace_all(result, regex::NoExpand(value)).into_owned();
        }
    }
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

    #[test]
    fn asymmetric_whitespace_all_match() {
        // Common typo cases: space on one side only, tabs, multi-space.
        let cases = [
            "{{globals.x}}",
            "{{ globals.x }}",
            "{{ globals.x}}",    // left-only
            "{{globals.x }}",    // right-only
            "{{  globals.x  }}", // multi-space
            "{{\tglobals.x\t}}", // tabs
        ];
        for c in cases {
            let result = substitute_vars(c, map(&[]), None, None, Some(map(&[("x", "Y")])));
            assert_eq!(result, "Y", "asymmetric form failed: {c:?}");
        }
    }
}
