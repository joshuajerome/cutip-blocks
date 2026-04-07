# CUTIP Blocks — Claude Working Context

## What is CUTIP Blocks

Reusable workflow blocks for CUTIP. Each block is a decorated Python function that maps to a single CLI operation (kubectl, ssh, cp, etc.). Blocks execute at runtime and provide metadata for cutip-desktop visualization.

## Key Conventions

- **Never auto-commit.** Only commit when explicitly asked.
- **uv** is the package manager. Always `uv run`, `uv add`.
- **Run tests**: `uv run pytest tests/ -v`
- **Block naming**: `category.verb` — `k8s.get_secret`, `ssh.exec`, `file.copy`
- **Each block function gets only the `@block` decorator** — no stacking
- **SSH sessions use `sesh` parameter name** — never `conn`, `session`, `ssh`
- **Logging**: every block logs its command via loguru with `[BlockName]` prefix
- **Redaction**: passwords/tokens/keys are never logged — use `****`

## Architecture

```
cutip_blocks/
├── __init__.py          # Public API: block decorator, BlockRegistry
├── decorator.py         # @block decorator + BlockMeta dataclass
├── registry.py          # BlockRegistry — discovers all blocks
├── utils.py             # is_empty helper
└── blocks/              # Block implementations by category
    ├── container.py     # start, stop, remove, exec, exec_stream
    ├── ssh.py           # connect (context manager), SSHSession.exec/probe
    ├── kubectl.py       # connect → KubectlSession (bound to SSH + namespace)
    ├── file.py          # copy, copy_tree, read_yaml, write_yaml, replace, is_empty
    ├── download.py      # http_fetch
    ├── config.py        # render_template, substitute_vars
    ├── validate.py      # path_exists, env_var_set, ip_valid
    ├── service.py       # poll_until_ready, wait_for_exit
    ├── crictl.py        # image_ls, image_rm, image_import
    └── ctr.py           # containerd ctr operations
```

## SSH + kubectl Pattern

All remote operations share a persistent SSH session. kubectl operations bind to the SSH session and a namespace:

```python
from cutip_blocks.blocks import container, ssh, kubectl

with ssh.connect(ctx, container="ops-runner",
                 host="10.0.0.1", username="root", password=pw) as sesh:
    sesh.probe()

    kube = kubectl.connect(sesh, namespace="prod")
    kube.get("deployment", name="web")
    kube.get_secret_value(secret="db-creds", key="password")
    kube.exec(target="deploy/web", cmd="cat /app/config.py")
    kube.patch_deployment(
        deployment="web",
        volume_name="config-override",
        host_path="/root/patches/config.py",
        mount_path="/opt/app/config.py",
    )
    kube.patch_file_from_pod(
        deployment="web",
        source_file="/opt/app/handler.py",
        dest_dir="/root/patches",
        replacements={"old_value": "new_value"},
    )
```

## KubectlSession Methods

| Method | Description |
|--------|-------------|
| `get(resource, name)` | `kubectl get <resource> <name> -o <format>` |
| `get_secret_value(secret, key)` | Decode a base64 field from a K8s secret |
| `find_pod(name_prefix)` | Find a pod by deployment name prefix, prefer Running |
| `exec(target, cmd)` | `kubectl exec <target> -- bash -c '<cmd>'` |
| `cat_file(target, path)` | Read a file from inside a pod (no tar needed) |
| `cp(pod, src, dest)` | `kubectl cp <pod>:<src> <dest>` |
| `apply(file)` | `kubectl apply -f <file>` |
| `rollout_status(resource, timeout)` | Wait for rollout to complete |
| `patch_deployment(deployment, volume_name, host_path, mount_path)` | Add hostPath volume + mount, apply, wait for rollout |
| `patch_file_from_pod(deployment, source_file, dest_dir, replacements)` | Copy file from pod, sed replacements, chmod (re-run safe) |

## Block Categories

| Category | Tool | Examples |
|----------|------|---------|
| container | podman/docker | start, stop, remove, exec |
| ssh | paramiko | connect (context manager), SSHSession.exec/probe |
| kubectl | kubectl over SSH | KubectlSession — get, exec, cp, apply, patch_deployment, patch_file_from_pod |
| file | filesystem | copy, copy_tree, read_yaml, write_yaml, replace |
| crictl | crictl/ctr | image_ls, image_rm, image_import |
| download | http | http_fetch |
| config | templates | render_template |
| validate | checks | path_exists, env_var_set |
| service | daemons | poll_until_ready |

## Branch Conventions

Same as cutip — see cutip/CLAUDE.md.

## Versioning

Semantic versioning. Stays at 0.x until cutip-core reaches 1.0.
