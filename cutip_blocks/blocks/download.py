"""Download blocks — HTTP fetch operations."""

from __future__ import annotations

import urllib.request
from pathlib import Path

from loguru import logger

from cutip_blocks.decorator import block


@block(name="HTTP Fetch", category="download", action="http_fetch")
def http_fetch(
    ctx,
    *,
    url: str,
    dest: str | Path,
    skip_if_exists: bool = True,
    timeout: int = 60,
    chunk_size: int = 8192,
    method: str = "GET",
) -> Path:
    """Download a file from a URL.

    Args:
        url: Remote URL to fetch.
        dest: Local file path to save to.
        skip_if_exists: Skip download if dest already exists and is non-empty.
        timeout: Request timeout in seconds.
        chunk_size: Read chunk size in bytes.
        method: HTTP method (GET or HEAD). HEAD only checks reachability.
    """
    dest = Path(dest)

    if method.upper() == "HEAD":
        logger.info("[HTTP Fetch] HEAD {}", url)
        req = urllib.request.Request(url, method="HEAD")
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            logger.info("[HTTP Fetch] {} → {}", url, resp.status)
        return dest

    if skip_if_exists and dest.exists() and dest.stat().st_size > 0:
        logger.info("[HTTP Fetch] Already exists: {}", dest)
        return dest

    logger.info("[HTTP Fetch] {} → {}", url, dest)
    dest.parent.mkdir(parents=True, exist_ok=True)

    req = urllib.request.Request(url)
    with urllib.request.urlopen(req, timeout=timeout) as resp, dest.open("wb") as fh:
        while True:
            chunk = resp.read(chunk_size)
            if not chunk:
                break
            fh.write(chunk)

    size_kb = dest.stat().st_size / 1024
    logger.info("[HTTP Fetch] Saved: {} ({:.0f} KB)", dest.name, size_kb)
    return dest
