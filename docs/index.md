# CUTIP Blocks

Reusable workflow blocks for [CUTIP](https://github.com/joshuajerome/cutip).

Each block is a `@block`-decorated Python function that maps to a single CLI operation (`kubectl`, `ssh`, `cp`, etc.). Blocks provide:

- **Runtime execution** — called from CUTIP workflows like any Python function
- **Metadata for visualization** — cutip-desktop parses block metadata to render DAG pipelines
- **Structured logging** — every block logs its command with `[BlockName]` prefix and credential redaction

## Quick Start

```bash
pip install cutip-blocks
```

```python
from cutip.workflow import action, orchestrator, stage
from cutip_blocks.blocks import container, ssh, k8s

@orchestrator
def main(ctx):
    container.start(ctx, container="my-app")

    with ssh.session(ctx, container="my-app",
                     host="10.0.0.1", username="root",
                     password=ctx.config["password"]) as sesh:
        stage("Validation")
        k8s.get_deployment(ctx, sesh, namespace="default", deployment="web")

    container.stop(ctx, container="my-app")
```

## Categories

| Category | Blocks | CLI Tool |
|----------|--------|----------|
| [Container](blocks/container.md) | `start`, `stop`, `remove`, `exec`, `exec_stream` | podman/docker |
| [SSH](blocks/ssh.md) | `session`, `exec`, `probe` | ssh/paramiko |
| [File](blocks/file.md) | `copy`, `copy_tree`, `read_yaml`, `write_yaml`, `read_json`, `write_json`, `replace`, `is_empty` | filesystem |
| [Kubernetes](blocks/k8s.md) | `get_deployment`, `get_secret`, `get_pod`, `exec`, `cp`, `apply`, `patch_deployment`, `rollout_status` | kubectl |
| [crictl](blocks/crictl.md) | `image_ls`, `image_rm`, `image_import` | crictl/ctr |
| [Download](blocks/download.md) | `http_fetch` | HTTP |
| [Config](blocks/config.md) | `render_template`, `substitute_vars` | templates |
| [Validate](blocks/validate.md) | `path_exists`, `env_var_set`, `ip_valid` | checks |
| [Service](blocks/service.md) | `poll_until_ready`, `wait_for_exit` | polling |
