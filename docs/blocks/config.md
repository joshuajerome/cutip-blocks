# config

Blocks in the `config` category.

## `render_template`

**Render Template**

```python
config.render_template(ctx, *, template: 'str | Path', dest: 'str | Path', variables: 'dict[str, str]') -> 'Path'
```

Render a template file with {{ var }} substitution.

Args:
    template: Path to template file.
    dest: Path to write rendered output.
    variables: Dict of variable name → value.

---

## `substitute_vars`

**Substitute Vars**

```python
config.substitute_vars(ctx, *, text: 'str', variables: 'dict[str, str]') -> 'str'
```

Replace {{ var }} placeholders in a string.

Args:
    text: Input string with placeholders.
    variables: Dict of variable name → value.

---
