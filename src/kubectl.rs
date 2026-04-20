//! kubectl session — Kubernetes operations over SSH.
//!
//! `KubectlSession` binds to an `SSHSession` and a default namespace.
//! Every method formats a `kubectl` command and dispatches it via SSH.

use std::collections::HashMap;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::ssh::{ExecResult, SSHSession};

/// Bound kubectl session over an SSH connection.
///
/// Created via `kubectl_connect(sesh, namespace="prod")`.
/// Every method runs a `kubectl` command over the bound SSH session.
#[pyclass]
pub struct KubectlSession {
    sesh: Py<SSHSession>,
    namespace: String,
}

impl KubectlSession {
    fn exec_ssh(&self, py: Python<'_>, cmd: &str) -> PyResult<ExecResult> {
        self.sesh.borrow_mut(py).run_cmd(py, cmd)
    }

    fn ns<'a>(&'a self, override_ns: Option<&'a str>) -> &'a str {
        override_ns.unwrap_or(&self.namespace)
    }
}

#[pymethods]
impl KubectlSession {
    /// Run `kubectl get <resource> -n <ns> <name> -o <format>`.
    #[pyo3(signature = (resource, *, name, namespace = None, output_format = "name"))]
    fn get(
        &self,
        py: Python<'_>,
        resource: &str,
        name: &str,
        namespace: Option<&str>,
        output_format: &str,
    ) -> PyResult<ExecResult> {
        let ns = self.ns(namespace);
        let cmd = format!("kubectl get {resource} -n {ns} {name} -o {output_format}");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "{resource} '{name}' not found in namespace {ns}: {}",
                result.stderr.trim()
            )));
        }
        Ok(result)
    }

    /// Decode a base64-encoded field from a Kubernetes secret.
    #[pyo3(signature = (*, secret, key, namespace = None))]
    fn get_secret_value(
        &self,
        py: Python<'_>,
        secret: &str,
        key: &str,
        namespace: Option<&str>,
    ) -> PyResult<String> {
        let ns = self.ns(namespace);
        let cmd = format!(
            "kubectl get secret -n {ns} {secret} -o jsonpath=\"{{.data.{key}}}\" | base64 -d"
        );
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "Failed to decode secret '{secret}' key '{key}' in {ns}: {}",
                result.stderr.trim()
            )));
        }
        Ok(result.stdout.trim().to_string())
    }

    /// Find a pod by name prefix. Prefers Running pods.
    #[pyo3(signature = (*, name_prefix, namespace = None))]
    fn find_pod(
        &self,
        py: Python<'_>,
        name_prefix: &str,
        namespace: Option<&str>,
    ) -> PyResult<String> {
        let ns = self.ns(namespace);
        let cmd = format!("kubectl get pod -n {ns} -o json");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "Failed to list pods in namespace {ns}: {}",
                result.stderr.trim()
            )));
        }

        let data: serde_json::Value = serde_json::from_str(&result.stdout)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse pod list JSON: {e}")))?;

        let items = data["items"]
            .as_array()
            .ok_or_else(|| PyRuntimeError::new_err("No 'items' array in pod list"))?;

        let mut matches: Vec<(String, String)> = Vec::new();
        for pod in items {
            let pod_name = pod["metadata"]["name"].as_str().unwrap_or("");
            if pod_name.starts_with(name_prefix) {
                let phase = pod["status"]["phase"].as_str().unwrap_or("Unknown");
                matches.push((pod_name.to_string(), phase.to_string()));
            }
        }

        if matches.is_empty() {
            return Err(PyRuntimeError::new_err(format!(
                "No pods matching '{name_prefix}' in namespace {ns}"
            )));
        }

        // Prefer Running pods
        let selected = matches
            .iter()
            .find(|(_, phase)| phase == "Running")
            .unwrap_or(&matches[0]);

        if matches.len() > 1 {
            dim_log!(
                "[kubectl find pod] Multiple pods match '{}': {}. Using: {} ({})",
                name_prefix,
                matches
                    .iter()
                    .map(|(n, p)| format!("{n} ({p})"))
                    .collect::<Vec<_>>()
                    .join(", "),
                selected.0,
                selected.1,
            );
        } else {
            dim_log!("[kubectl find pod] Found: {} ({})", selected.0, selected.1);
        }

        Ok(selected.0.clone())
    }

    /// Run `kubectl exec <target> -- bash -c '<cmd>'`.
    #[pyo3(signature = (*, target, cmd, namespace = None))]
    fn exec(
        &self,
        py: Python<'_>,
        target: &str,
        cmd: &str,
        namespace: Option<&str>,
    ) -> PyResult<ExecResult> {
        let ns = self.ns(namespace);
        let full_cmd = format!("kubectl exec -n {ns} {target} -- bash -c '{cmd}'");
        self.exec_ssh(py, &full_cmd)
    }

    /// Read a file from inside a pod (no tar needed).
    #[pyo3(signature = (*, target, path, namespace = None))]
    fn cat_file(
        &self,
        py: Python<'_>,
        target: &str,
        path: &str,
        namespace: Option<&str>,
    ) -> PyResult<String> {
        let ns = self.ns(namespace);
        let cmd = format!("kubectl exec -n {ns} {target} -- cat {path}");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "Failed to read {path} from {target}: {}",
                result.stderr.trim()
            )));
        }
        Ok(result.stdout)
    }

    /// Run `kubectl cp <ns>/<pod>:<src> <dest>`.
    #[pyo3(signature = (*, pod, src, dest, namespace = None))]
    fn cp(
        &self,
        py: Python<'_>,
        pod: &str,
        src: &str,
        dest: &str,
        namespace: Option<&str>,
    ) -> PyResult<ExecResult> {
        let ns = self.ns(namespace);
        let cmd = format!("kubectl cp {ns}/{pod}:{src} {dest}");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "kubectl cp failed: {}",
                result.stderr.trim()
            )));
        }
        Ok(result)
    }

    /// Run `kubectl apply -f <file>`.
    #[pyo3(signature = (*, file))]
    fn apply(&self, py: Python<'_>, file: &str) -> PyResult<ExecResult> {
        let cmd = format!("kubectl apply -f {file}");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "kubectl apply failed: {}",
                result.stderr.trim()
            )));
        }
        Ok(result)
    }

    /// Wait for a rollout to complete.
    #[pyo3(signature = (*, resource, namespace = None, timeout = 120))]
    fn rollout_status(
        &self,
        py: Python<'_>,
        resource: &str,
        namespace: Option<&str>,
        timeout: u32,
    ) -> PyResult<ExecResult> {
        let ns = self.ns(namespace);
        let cmd = format!("kubectl rollout status {resource} -n {ns} --timeout={timeout}s");
        let result = self.exec_ssh(py, &cmd)?;
        if result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "Rollout failed for {resource} in {ns}: {}",
                result.stderr.trim()
            )));
        }
        dim_log!("[kubectl rollout status] {resource} rolled out successfully");
        Ok(result)
    }

    /// Patch a deployment to add a hostPath volume + volumeMount.
    #[pyo3(signature = (*, deployment, volume_name, host_path, mount_path, namespace = None, rollout_timeout = 120))]
    fn patch_deployment(
        &self,
        py: Python<'_>,
        deployment: &str,
        volume_name: &str,
        host_path: &str,
        mount_path: &str,
        namespace: Option<&str>,
        rollout_timeout: u32,
    ) -> PyResult<ExecResult> {
        let ns = self.ns(namespace);
        let remote_patch_file = format!("/tmp/{deployment}-patched.yaml");

        // Fetch current deployment YAML
        dim_log!("[kubectl patch deployment] Fetching {ns}/{deployment} ...");
        let get_cmd = format!("kubectl get deployment -n {ns} {deployment} -o yaml");
        let result = self.exec_ssh(py, &get_cmd)?;
        if result.exit_code != 0 || result.stdout.trim().is_empty() {
            return Err(PyRuntimeError::new_err(format!(
                "Failed to get deployment {deployment} in {ns}: {}",
                result.stderr.trim()
            )));
        }

        // Parse and patch YAML in Rust
        let patched_yaml =
            patch_deployment_yaml(&result.stdout, volume_name, host_path, mount_path)?;

        // Write patched YAML to remote
        let write_cmd =
            format!("cat << 'PATCH_EOF' > {remote_patch_file}\n{patched_yaml}\nPATCH_EOF");
        self.exec_ssh(py, &write_cmd)?;

        // Apply
        let apply_cmd = format!("kubectl apply -f {remote_patch_file}");
        let apply_result = self.exec_ssh(py, &apply_cmd)?;

        // Cleanup
        self.exec_ssh(py, &format!("rm -f {remote_patch_file}"))?;

        if apply_result.exit_code != 0 {
            return Err(PyRuntimeError::new_err(format!(
                "kubectl apply failed: {}",
                apply_result.stderr.trim()
            )));
        }

        // Wait for rollout
        self.rollout_status(
            py,
            &format!("deployment/{deployment}"),
            namespace,
            rollout_timeout,
        )
    }

    /// Copy a file from a pod, apply sed replacements, chmod. Re-run safe.
    #[pyo3(signature = (*, deployment, source_file, dest_dir, replacements, chmod = "777", namespace = None))]
    fn patch_file_from_pod(
        &self,
        py: Python<'_>,
        deployment: &str,
        source_file: &str,
        dest_dir: &str,
        replacements: HashMap<String, String>,
        chmod: &str,
        namespace: Option<&str>,
    ) -> PyResult<String> {
        let ns = self.ns(namespace);
        let source_basename = source_file.rsplit_once('/').map_or(source_file, |s| s.1);
        let target_file = format!("{dest_dir}/{source_basename}");
        let backup = format!("{target_file}.bak");

        // Create destination directory
        self.exec_ssh(py, &format!("mkdir -p {dest_dir}"))?;

        // Back up existing file
        let has_existing = self
            .exec_ssh(py, &format!("test -s {target_file}"))?
            .exit_code
            == 0;
        if has_existing {
            dim_log!("[kubectl patch file] Backing up {target_file} → {backup}");
            self.exec_ssh(py, &format!("cp {target_file} {backup}"))?;
        }

        // Re-run safe: use backup if available
        let has_backup = self.exec_ssh(py, &format!("test -s {backup}"))?.exit_code == 0;

        if has_backup {
            dim_log!("[kubectl patch file] Re-run detected — restoring from {backup}");
            self.exec_ssh(py, &format!("cp {backup} {target_file}"))?;
        } else {
            // First run: verify source exists, copy from pod
            let verify = self.exec_ssh(
                py,
                &format!("kubectl exec -n {ns} deploy/{deployment} -- test -f {source_file}"),
            )?;
            if verify.exit_code != 0 {
                return Err(PyRuntimeError::new_err(format!(
                    "Source file not found in pod: {source_file}"
                )));
            }

            let temp_file = format!("{target_file}.tmp");
            dim_log!("[kubectl patch file] Copying {source_file} from pod → {target_file}");
            let copy_cmd = format!(
                "bash -c 'kubectl exec -n {ns} deploy/{deployment} -- cat {source_file} > {temp_file}'"
            );
            self.exec_ssh(py, &copy_cmd)?;
            self.exec_ssh(py, &format!("mv {temp_file} {target_file}"))?;
        }

        // Sanity check
        let check = self.exec_ssh(
            py,
            &format!("test -s {target_file} && head -c 4 {target_file}"),
        )?;
        if check.exit_code != 0 || check.stdout.trim().is_empty() {
            return Err(PyRuntimeError::new_err(format!(
                "Copied file is missing or empty: {target_file}"
            )));
        }

        // Apply sed replacements
        for (original, patched) in &replacements {
            dim_log!(
                "[kubectl patch file] sed: {}... → {}...",
                &original[..original.len().min(50)],
                &patched[..patched.len().min(50)]
            );
            let escaped_orig = original.replace('\'', "'\\''");
            let escaped_patched = patched.replace('\'', "'\\''");
            self.exec_ssh(
                py,
                &format!("sed -i 's|{escaped_orig}|{escaped_patched}|g' {target_file}"),
            )?;
        }

        // Set permissions
        dim_log!("[kubectl patch file] chmod {chmod} {target_file}");
        self.exec_ssh(py, &format!("chmod {chmod} {target_file}"))?;

        dim_log!("[kubectl patch file] Patched and ready: {target_file}");
        Ok(target_file)
    }
}

/// Parse deployment YAML, inject volume + volumeMount, strip metadata.
fn patch_deployment_yaml(
    raw_yaml: &str,
    volume_name: &str,
    host_path: &str,
    mount_path: &str,
) -> PyResult<String> {
    let mut doc: serde_yaml::Value = serde_yaml::from_str(raw_yaml)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse deployment YAML: {e}")))?;

    let spec = doc
        .get_mut("spec")
        .and_then(|s| s.get_mut("template"))
        .and_then(|s| s.get_mut("spec"))
        .ok_or_else(|| PyRuntimeError::new_err("Missing spec.template.spec in deployment"))?;

    // Inject volume
    let volumes = spec.get_mut("volumes").and_then(|v| v.as_sequence_mut());

    let volume_entry = serde_yaml::to_value(serde_json::json!({
        "name": volume_name,
        "hostPath": { "path": host_path, "type": "" }
    }))
    .unwrap();

    if let Some(volumes) = volumes {
        if !volumes
            .iter()
            .any(|v| v.get("name").and_then(|n| n.as_str()) == Some(volume_name))
        {
            volumes.push(volume_entry);
            dim_log!("[kubectl patch deployment] Added volume '{volume_name}'");
        } else {
            dim_log!("[kubectl patch deployment] Volume '{volume_name}' already present");
        }
    } else {
        spec["volumes"] = serde_yaml::Value::Sequence(vec![volume_entry]);
        dim_log!("[kubectl patch deployment] Added volume '{volume_name}'");
    }

    // Inject volumeMount on first container
    let containers = spec
        .get_mut("containers")
        .and_then(|c| c.as_sequence_mut())
        .ok_or_else(|| PyRuntimeError::new_err("No containers in deployment spec"))?;

    let container = containers
        .first_mut()
        .ok_or_else(|| PyRuntimeError::new_err("Empty containers list"))?;

    let mount_entry = serde_yaml::to_value(serde_json::json!({
        "name": volume_name,
        "mountPath": mount_path
    }))
    .unwrap();

    let mounts = container
        .get_mut("volumeMounts")
        .and_then(|v| v.as_sequence_mut());

    if let Some(mounts) = mounts {
        if !mounts
            .iter()
            .any(|m| m.get("name").and_then(|n| n.as_str()) == Some(volume_name))
        {
            mounts.push(mount_entry);
            dim_log!("[kubectl patch deployment] Added volumeMount '{volume_name}'");
        } else {
            dim_log!("[kubectl patch deployment] VolumeMount '{volume_name}' already present");
        }
    } else {
        container["volumeMounts"] = serde_yaml::Value::Sequence(vec![mount_entry]);
        dim_log!("[kubectl patch deployment] Added volumeMount '{volume_name}'");
    }

    // Strip fields that block kubectl apply
    if let Some(metadata) = doc.get_mut("metadata") {
        if let Some(map) = metadata.as_mapping_mut() {
            for key in &["resourceVersion", "uid", "creationTimestamp", "generation"] {
                map.remove(serde_yaml::Value::String(key.to_string()));
            }
        }
        if let Some(annotations) = metadata.get_mut("annotations") {
            if let Some(map) = annotations.as_mapping_mut() {
                map.remove(serde_yaml::Value::String(
                    "kubectl.kubernetes.io/last-applied-configuration".to_string(),
                ));
            }
        }
    }
    if let Some(map) = doc.as_mapping_mut() {
        map.remove(serde_yaml::Value::String("status".to_string()));
    }

    serde_yaml::to_string(&doc)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize patched YAML: {e}")))
}

/// Create a bound kubectl session over an existing SSH connection.
#[pyfunction]
#[pyo3(signature = (sesh, *, namespace))]
pub fn kubectl_connect(sesh: Py<SSHSession>, namespace: &str) -> KubectlSession {
    dim_log!("[kubectl] Session bound to namespace '{namespace}'");
    KubectlSession {
        sesh,
        namespace: namespace.to_string(),
    }
}
