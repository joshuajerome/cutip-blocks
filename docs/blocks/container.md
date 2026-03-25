# container

Blocks in the `container` category.

## `exec`

**Exec in Container**

```python
container.exec(ctx, *, container: 'str', cmd: 'str | list[str]', detach: 'bool' = False)
```

Execute a command inside a container.

Args:
    container: Container name.
    cmd: Command string or list.
    detach: If True, fire and forget.

---

## `exec_stream`

**Exec Stream**

```python
container.exec_stream(ctx, *, container: 'str', cmd: 'str | list[str]')
```

Execute a command inside a container with streaming output.

Streams stdout to sys.stdout in real time.

---

## `remove`

**Remove Container**

```python
container.remove(ctx, *, container: 'str')
```

Remove a container.

---

## `start`

**Start Container**

```python
container.start(ctx, *, container: 'str')
```

Start a container by name.

---

## `stop`

**Stop Container**

```python
container.stop(ctx, *, container: 'str')
```

Stop a running container.

---
