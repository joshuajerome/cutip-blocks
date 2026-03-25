# service

Blocks in the `service` category.

## `poll_until_ready`

**Poll Until Ready**

```python
service.poll_until_ready(ctx, *, check_fn, timeout: 'int' = 30, interval: 'int' = 1, description: 'str' = 'service') -> 'bool'
```

Poll a callable until it returns truthy or timeout is reached.

Args:
    check_fn: Callable that returns truthy when ready.
    timeout: Max wait time in seconds.
    interval: Seconds between polls.
    description: Label for log messages.

---

## `wait_for_exit`

**Wait for Exit**

```python
service.wait_for_exit(ctx, *, container_obj, timeout: 'int' = 60, interval: 'float' = 0.5) -> 'int'
```

Wait for a container to exit and return exit code.

Args:
    container_obj: Docker/Podman container object with .reload() and .status.
    timeout: Max wait time in seconds.
    interval: Seconds between polls.

---
