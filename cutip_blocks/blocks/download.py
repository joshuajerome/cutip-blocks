"""Backward compat — download.http_fetch maps to http.get"""
from cutip_blocks.http import get as http_fetch

__all__ = ["http_fetch"]
