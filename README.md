# cutip-blocks

[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/rust-PyO3-orange)](https://pyo3.rs/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

Rust-backed workflow blocks for [CUTIP](https://github.com/joshuajerome/cutip). Write Python, execute Rust.

Every function you call from `cutip_blocks` is a compiled Rust binary under the hood — SSH via [russh](https://crates.io/crates/russh), Docker/Podman via [bollard](https://crates.io/crates/bollard), HTTP via [reqwest](https://crates.io/crates/reqwest), YAML/JSON via [serde](https://crates.io/crates/serde). Zero Python dependencies. The Python GIL is released during all I/O.

## Install

```bash
pip install cutip-blocks
```

## Modules

| Module | Rust crate | What it does |
|--------|-----------|-------------|
| **ssh** | russh | Persistent SSH sessions — `connect()`, `exec()`, `probe()` |
| **kubectl** | serde | Kubernetes ops over SSH — `get`, `exec`, `find_pod`, `patch_deployment`, `patch_file_from_pod` |
| **container** | bollard | Docker/Podman — `start`, `stop`, `remove`, `exec`, `pull` |
| **file** | std::fs + serde | `copy`, `copy_tree`, `read/write_json`, `read/write_yaml`, `replace` |
| **http** | reqwest | `get`, `post`, `put`, `delete` with JSON + TLS toggle |
| **network** | bollard | `create`, `remove`, `exists` |
| **service** | reqwest + bollard | `poll_until_ready`, `wait_for_exit` |
| **validate** | std | `path_exists`, `env_var_set`, `ip_valid` |
| **config** | string ops | `render_template`, `substitute_vars` |

## Usage

### SSH + kubectl (remote VM operations)

```python
from cutip_blocks import ssh, kubectl

with ssh.connect(host="10.0.0.1", username="root", password=pw) as sesh:
    sesh.probe()  # verify connectivity

    kube = kubectl.connect(sesh, namespace="prod")
    kube.get("deployment", name="web")
    kube.find_pod(name_prefix="web")

    password = kube.get_secret_value(secret="db-creds", key="password")
    kube.exec(target="deploy/web", cmd="cat /app/config.py")

    kube.patch_file_from_pod(
        deployment="web",
        source_file="/opt/app/handler.py",
        dest_dir="/root/patches",
        replacements={"old_value": "new_value"},
    )

    kube.patch_deployment(
        deployment="web",
        volume_name="config-override",
        host_path="/root/patches/handler.py",
        mount_path="/opt/app/handler.py",
    )
```

### Container management

```python
from cutip_blocks import container

rt = container.connect()  # auto-detect Docker/Podman
rt.pull("nginx", tag="latest")
rt.start("my-container")
result = rt.exec("my-container", "nginx -t")
print(result.stdout)
rt.stop("my-container")
rt.remove("my-container")
```

### File operations

```python
from cutip_blocks import file

data = file.read_json("config.json")
file.copy_tree("src/", "build/deps/")
file.replace("config.yaml", "old_value", "new_value")
file.write_yaml("output.yaml", {"key": "value"})
```

### HTTP

```python
from cutip_blocks import http

resp = http.post("https://api.example.com/token",
                 json={"username": "admin", "password": "secret"},
                 verify_tls=False)
token = resp.json()["access_token"]
```

## Development

Requires [Rust toolchain](https://rustup.rs/) + Python 3.11+.

```bash
git clone https://github.com/joshuajerome/cutip-blocks.git
cd cutip-blocks
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
pytest tests/ -v
```

## Ecosystem

| Project | Description |
|---------|-------------|
| [cutip](https://github.com/joshuajerome/cutip) | Workflow automation framework |
| [cutip-blocks](https://github.com/joshuajerome/cutip-blocks) | Rust-backed blocks (this repo) |
| [cutip-desktop](https://github.com/joshuajerome/cutip-desktop) | Visual companion — DAG, container management |

## License

MIT
