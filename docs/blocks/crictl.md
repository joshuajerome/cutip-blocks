# crictl

Blocks in the `crictl` category.

## `image_import`

**Image Import**

```python
crictl.image_import(ctx, sesh: 'SSHSession', *, tar: 'str', socket: 'str' = '/run/k3s/containerd/containerd.sock', namespace: 'str' = 'k8s.io') -> 'ExecResult'
```

Import a container image tarball via ctr.

Args:
    tar: Path to the .tar image on the remote host.
    socket: Containerd socket path.
    namespace: Containerd namespace.

---

## `image_ls`

**Image List**

```python
crictl.image_ls(ctx, sesh: 'SSHSession') -> 'ExecResult'
```

List container images via crictl.

---

## `image_rm`

**Image Remove**

```python
crictl.image_rm(ctx, sesh: 'SSHSession', *, image: 'str') -> 'ExecResult'
```

Remove a container image via crictl.

---
