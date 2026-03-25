# download

Blocks in the `download` category.

## `http_fetch`

**HTTP Fetch**

```python
download.http_fetch(ctx, *, url: 'str', dest: 'str | Path', skip_if_exists: 'bool' = True, timeout: 'int' = 60, chunk_size: 'int' = 8192, method: 'str' = 'GET') -> 'Path'
```

Download a file from a URL.

Args:
    url: Remote URL to fetch.
    dest: Local file path to save to.
    skip_if_exists: Skip download if dest already exists and is non-empty.
    timeout: Request timeout in seconds.
    chunk_size: Read chunk size in bytes.
    method: HTTP method (GET or HEAD). HEAD only checks reachability.

---
