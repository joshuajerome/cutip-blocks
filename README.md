# cutip-blocks

[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-blue)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![cutip](https://img.shields.io/badge/cutip-%E2%89%A50.2.0-purple)](https://github.com/joshuajerome/cutip)

Reusable workflow blocks for [CUTIP](https://github.com/joshuajerome/cutip) — Container Unit Templates in Python.

Each block is a decorated Python function that maps to a single CLI operation (`kubectl`, `ssh`, `cp`, etc.). Blocks execute at runtime and provide metadata for [cutip-desktop](https://github.com/joshuajerome/cutip-desktop) DAG visualization.

## Installation

```bash
pip install cutip-blocks
```

## Block Categories

| Category | Blocks | Maps to |
|----------|--------|---------|
| **container** | `start`, `stop`, `remove`, `exec`, `exec_stream` | `podman`/`docker` |
| **ssh** | `session`, `exec`, `probe` | `ssh`/`paramiko` |
| **file** | `copy`, `copy_tree`, `read_yaml`, `write_yaml`, `read_json`, `write_json`, `replace`, `is_empty` | filesystem |
| **k8s** | `get_deployment`, `get_secret`, `get_pod`, `exec`, `cp`, `apply`, `patch_deployment`, `rollout_status` | `kubectl` |
| **crictl** | `image_ls`, `image_rm`, `image_import` | `crictl`/`ctr` |
| **download** | `http_fetch` | HTTP GET |
| **config** | `render_template`, `substitute_vars` | template rendering |
| **validate** | `path_exists`, `env_var_set`, `ip_valid` | precondition checks |
| **service** | `poll_until_ready`, `wait_for_exit` | readiness polling |

## Usage

```python
from cutip.workflow import action, orchestrator, stage
from cutip_blocks.blocks import container, ssh, k8s

@action(name="Check Deployment")
def check_deploy(ctx, sesh, ns, deploy):
    k8s.get_deployment(ctx, sesh, namespace=ns, deployment=deploy)

@orchestrator
def main(ctx):
    container.start(ctx, container="my-app")

    with ssh.session(ctx, container="my-app",
                     host="10.0.0.1", username="root",
                     password=ctx.config["password"]) as sesh:

        stage("Validation")
        check_deploy(ctx, sesh, "default", "web")

        stage("Operations")
        k8s.apply(ctx, sesh, file="/tmp/patch.yaml")

    container.stop(ctx, container="my-app")
```

## SSH Session

All SSH-based blocks share a persistent connection via context manager. One SSH handshake, reused for all commands. Credentials are redacted in all log output.

```
INFO | [Get Deployment] ssh root@10.0.0.1 :: kubectl get deployment -n default web -o name
INFO | [Get Secret] ssh root@10.0.0.1 :: kubectl get secret -n ns creds -o name
```

## Block Discovery

```python
from cutip_blocks import BlockRegistry

registry = BlockRegistry.discover()
for meta, fn in registry.blocks:
    print(f"{meta.category}.{meta.action} — {meta.name}")
```

## License

MIT
