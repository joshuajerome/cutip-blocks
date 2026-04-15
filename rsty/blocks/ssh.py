"""Backward compat — re-exports from rsty.ssh"""
from rsty.ssh import *
from rsty.ssh import connect
from rsty._core import SSHSession, ExecResult, ssh_connect

session = connect  # legacy alias
