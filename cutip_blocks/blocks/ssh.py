"""Backward compat — re-exports from cutip_blocks.ssh"""
from cutip_blocks.ssh import *
from cutip_blocks.ssh import connect
from cutip_blocks._core import SSHSession, ExecResult, ssh_connect

session = connect  # legacy alias
