"""kubectl blocks — Python API wrapping Rust kubectl implementation."""

from __future__ import annotations

from cutip_blocks._core import KubectlSession, kubectl_connect
from cutip_blocks._core import SSHSession


def connect(sesh: SSHSession, *, namespace: str) -> KubectlSession:
    """Create a bound kubectl session over an existing SSH connection.

    Args:
        sesh: An active SSHSession (from ``ssh.connect()``).
        namespace: Default Kubernetes namespace for all commands.

    Returns:
        A KubectlSession bound to the SSH connection and namespace.

    Example::

        with ssh.connect(host="10.0.0.1", username="root", password=pw) as sesh:
            kube = kubectl.connect(sesh, namespace="prod")
            kube.get("deployment", name="web")
            kube.find_pod(name_prefix="web")
    """
    return kubectl_connect(sesh, namespace=namespace)
