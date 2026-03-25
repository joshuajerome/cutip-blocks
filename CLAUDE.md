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
└── blocks/              # Block implementations by category
    ├── container.py     # start, stop, remove, exec, exec_stream
    ├── ssh.py           # session, exec, probe (SSHSession with redaction)
    ├── file.py          # copy, copy_tree, read_yaml, write_yaml, replace, is_empty
    ├── k8s.py           # get_deployment, get_secret, get_pod, exec, cp, apply, patch_deployment, rollout_status
    ├── download.py      # http_fetch
    ├── config.py        # render_template, substitute_vars
    ├── validate.py      # path_exists, env_var_set, ip_valid
    ├── service.py       # poll_until_ready, wait_for_exit
    └── crictl.py        # image_ls, image_rm, image_import
```

## Block Categories

| Category | Tool | Examples |
|----------|------|---------|
| container | podman/docker | start, stop, remove, exec |
| ssh | ssh/paramiko | session, exec, probe |
| file | filesystem | copy, replace, is_empty |
| k8s | kubectl | get_secret, get_pod, cp, apply |
| crictl | crictl/ctr | image_ls, image_rm, image_import |
| download | http | http_fetch |
| config | templates | render_template |
| validate | checks | path_exists, env_var_set |
| service | daemons | poll_until_ready |

## Branch Conventions

Same as cutip — see cutip/CLAUDE.md.

## Versioning

Semantic versioning. Stays at 0.x until cutip-core reaches 1.0.
