# CUTIP Blocks — Claude Working Context

## What is CUTIP Blocks

Rust-backed workflow blocks for CUTIP. Each block is a Python function that calls compiled Rust code via PyO3. All I/O (SSH, HTTP, Docker API, filesystem) happens in Rust. Python is the call surface only. Zero Python dependencies.

## Key Conventions

- **Never auto-commit.** Only commit when explicitly asked.
- **Build**: `maturin develop` (requires Rust toolchain + maturin)
- **Run tests**: `pytest tests/ -v` (after maturin develop)
- **Block naming**: `module.function` — `ssh.connect`, `kubectl.connect`, `file.copy`
- **SSH sessions use `sesh`** — never `conn`, `session`
- **Logging**: every block logs via `eprintln!` with `[Module]` prefix
- **Redaction**: passwords are replaced with `****` in SSH log output

## Architecture

```
cutip-blocks/
├── Cargo.toml                  # Rust deps: pyo3, russh, bollard, reqwest, serde
├── pyproject.toml              # maturin build backend
├── src/                        # Rust implementation
│   ├── lib.rs                  # PyO3 module registration (48 exports)
│   ├── ssh.rs                  # SSHSession via russh + tokio
│   ├── kubectl.rs              # KubectlSession (10 methods, serde_json/yaml)
│   ├── container.rs            # ContainerRuntime via bollard
│   ├── file.rs                 # std::fs + serde_json + serde_yaml
│   ├── http.rs                 # reqwest
│   ├── network.rs              # bollard network API
│   ├── service.rs              # reqwest poll + bollard wait
│   ├── validate.rs             # std path/env/ip checks
│   └── config.rs               # string template replacement
├── cutip_blocks/               # Python wrappers (thin, call _core.so)
│   ├── __init__.py
│   ├── ssh.py                  # connect() context manager
│   ├── kubectl.py              # connect() wrapper
│   ├── container.py            # connect() wrapper
│   ├── file.py                 # re-exports
│   ├── http.py                 # re-exports
│   ├── network.py              # re-exports
│   ├── service.py              # re-exports
│   ├── validate.py             # re-exports
│   ├── config.py               # re-exports
│   ├── utils.py                # is_empty
│   └── blocks/                 # backward-compat layer for existing consumers
│       ├── ssh.py              # re-exports from cutip_blocks.ssh
│       ├── kubectl.py          # re-exports from cutip_blocks.kubectl
│       ├── container.py        # re-exports + legacy start/stop/remove(ctx, container=)
│       ├── file.py             # re-exports from cutip_blocks.file
│       └── download.py         # http_fetch → http.get compat
└── tests/                      # 27 tests
```

## SSH + kubectl Pattern

```python
from cutip_blocks import ssh, kubectl

with ssh.connect(host="10.0.0.1", username="root", password=pw) as sesh:
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
```

## Modules

| Module | Rust crate | Functions |
|--------|-----------|-----------|
| ssh | russh + tokio | `connect()`, `SSHSession.exec/probe/close` |
| kubectl | serde_json/yaml | `connect()`, `KubectlSession` (10 methods) |
| container | bollard | `connect()`, `ContainerRuntime.start/stop/remove/exec/pull/exists` |
| file | std::fs + serde | `copy`, `copy_tree`, `read/write_json`, `read/write_yaml`, `replace`, `is_empty` |
| http | reqwest | `get`, `post`, `put`, `delete` |
| network | bollard | `create`, `remove`, `exists` |
| service | reqwest + bollard | `poll_until_ready`, `wait_for_exit` |
| validate | std | `path_exists`, `env_var_set`, `ip_valid` |
| config | string ops | `render_template`, `substitute_vars` |

## KubectlSession Methods

| Method | Description |
|--------|-------------|
| `get(resource, name)` | `kubectl get <resource> <name> -o <format>` |
| `get_secret_value(secret, key)` | Decode a base64 field from a K8s secret |
| `find_pod(name_prefix)` | Find pod by prefix, prefer Running |
| `exec(target, cmd)` | `kubectl exec <target> -- bash -c '<cmd>'` |
| `cat_file(target, path)` | Read file from pod (no tar) |
| `cp(pod, src, dest)` | `kubectl cp` |
| `apply(file)` | `kubectl apply -f` |
| `rollout_status(resource, timeout)` | Wait for rollout |
| `patch_deployment(...)` | Inject hostPath volume + mount, apply, rollout |
| `patch_file_from_pod(...)` | Copy from pod, sed, chmod (re-run safe) |

## Branch Conventions

Same as cutip — see cutip/CLAUDE.md.

## Versioning

Semantic versioning. Stays at 0.x until cutip-core reaches 1.0.
