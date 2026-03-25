# ssh

Blocks in the `ssh` category.

## `exec`

**SSH Exec**

```python
ssh.exec(ctx, sesh: 'SSHSession', *, cmd: 'str') -> 'ExecResult'
```

Execute a command over an existing SSH session.

---

## `probe`

**SSH Probe**

```python
ssh.probe(ctx, sesh: 'SSHSession') -> 'ExecResult'
```

Verify SSH session is authenticated and responsive.

---
