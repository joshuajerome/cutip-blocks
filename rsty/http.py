"""HTTP blocks — Python API wrapping Rust reqwest."""

from rsty._core import (
    get,
    post,
    put,
    delete,
    HttpResponse,
)

__all__ = ["get", "post", "put", "delete", "HttpResponse"]
