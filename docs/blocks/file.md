# file

Blocks in the `file` category.

## `copy`

**Copy File**

```python
file.copy(ctx, *, src: 'str | Path', dest: 'str | Path') -> 'Path'
```

Copy a single file.

---

## `copy_tree`

**Copy Tree**

```python
file.copy_tree(ctx, *, src: 'str | Path', dest: 'str | Path', clean: 'bool' = False) -> 'Path'
```

Copy a directory recursively.

---

## `is_empty`

**Is Empty**

```python
file.is_empty(value: 'str | bytes | None') -> 'bool'
```

Check if a value (file content, command output) is empty or None.

---

## `read_json`

**Read JSON**

```python
file.read_json(ctx, *, path: 'str | Path') -> 'dict'
```

Read and parse a JSON file.

---

## `read_yaml`

**Read YAML**

```python
file.read_yaml(ctx, *, path: 'str | Path') -> 'dict'
```

Read and parse a YAML file.

---

## `replace`

**Replace in File**

```python
file.replace(ctx, *, path: 'str | Path', old: 'str', new: 'str') -> 'Path'
```

Replace a string in a file (all occurrences).

---

## `write_json`

**Write JSON**

```python
file.write_json(ctx, *, path: 'str | Path', data: 'dict') -> 'Path'
```

Write a dict to a JSON file.

---

## `write_yaml`

**Write YAML**

```python
file.write_yaml(ctx, *, path: 'str | Path', data: 'dict') -> 'Path'
```

Write a dict to a YAML file.

---
