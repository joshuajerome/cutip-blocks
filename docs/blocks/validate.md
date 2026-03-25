# validate

Blocks in the `validate` category.

## `env_var_set`

**Env Var Set**

```python
validate.env_var_set(ctx, *, key: 'str') -> 'str'
```

Check that an environment variable is set and non-empty. Returns value.

---

## `ip_valid`

**IP Valid**

```python
validate.ip_valid(ctx, *, address: 'str') -> 'bool'
```

Validate an IP address format.

---

## `path_exists`

**Path Exists**

```python
validate.path_exists(ctx, *, path: 'str | Path', description: 'str' = '') -> 'bool'
```

Check that a filesystem path exists.

---
