# k8s

Blocks in the `k8s` category.

## `apply`

**Apply**

```python
k8s.apply(ctx, sesh: 'SSHSession', *, file: 'str') -> 'ExecResult'
```

Apply a YAML manifest via kubectl apply -f.

---

## `cp`

**Copy from Pod**

```python
k8s.cp(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', src: 'str', dest: 'str') -> 'ExecResult'
```

Copy a file from a pod to the VM filesystem.

Resolves the pod name from the deployment label, then runs kubectl cp.

---

## `exec`

**Exec in Pod**

```python
k8s.exec(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', cmd: 'str') -> 'ExecResult'
```

Execute a command inside a pod via kubectl exec.

Routes through ``deploy/`` so kubectl picks the pod automatically.
Supports pipes: ``cmd="cat /app/config.py | head -5"``

---

## `get_deployment`

**Get Deployment**

```python
k8s.get_deployment(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', output: 'str' = 'name') -> 'ExecResult'
```

Verify a deployment exists. Raises on failure.

---

## `get_pod`

**Get Pod**

```python
k8s.get_pod(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', output: 'str' = "jsonpath='{.items[0].metadata.name}'") -> 'ExecResult'
```

Get pod(s) for a deployment by label selector.

---

## `get_secret`

**Get Secret**

```python
k8s.get_secret(ctx, sesh: 'SSHSession', *, namespace: 'str', secret: 'str', output: 'str' = 'name') -> 'ExecResult'
```

Get a Kubernetes secret. Use output='jsonpath=...' for specific fields.

Example — decode a password::

    result = k8s.get_secret(ctx, sesh, namespace="ns", secret="my-creds",
                            output='jsonpath="{.data.password}" | base64 -d')

---

## `patch_deployment`

**Patch Deployment**

```python
k8s.patch_deployment(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', volume_name: 'str', host_path: 'str', mount_path: 'str') -> 'ExecResult'
```

Patch a deployment to add a hostPath volume mount.

1. Fetches current deployment YAML
2. Adds/updates the volume + volumeMount
3. Applies the patched manifest
4. Waits for rollout

---

## `rollout_status`

**Rollout Status**

```python
k8s.rollout_status(ctx, sesh: 'SSHSession', *, namespace: 'str', deployment: 'str', timeout: 'int' = 120) -> 'ExecResult'
```

Wait for a deployment rollout to complete.

---
