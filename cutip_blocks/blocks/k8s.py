"""k8s blocks — DEPRECATED. Use ``kubectl`` module instead."""

from __future__ import annotations

import warnings

from cutip_blocks.blocks.kubectl import KubectlSession
from cutip_blocks.blocks.ssh import ExecResult, SSHSession
from cutip_blocks.decorator import block


def _warn(old: str, new: str) -> None:
    warnings.warn(f"k8s.{old}() is deprecated. Use {new}.", DeprecationWarning, stacklevel=3)


@block(name="Get Deployment", category="k8s", action="get_deployment")
def get_deployment(
    ctx, sesh: SSHSession, *, namespace: str, deployment: str, output: str = "name"
) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.get("deployment", name=...)``."""
    _warn("get_deployment", 'kube.get("deployment", name=...)')
    return KubectlSession(sesh, namespace).get("deployment", name=deployment, output_format=output)


@block(name="Get Secret", category="k8s", action="get_secret")
def get_secret(
    ctx, sesh: SSHSession, *, namespace: str, secret: str, output: str = "name"
) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.get("secret", name=...)``."""
    _warn("get_secret", 'kube.get("secret", name=...)')
    return KubectlSession(sesh, namespace).get("secret", name=secret, output_format=output)


@block(name="Get Pod", category="k8s", action="get_pod")
def get_pod(
    ctx, sesh: SSHSession, *, namespace: str, deployment: str, output: str | None = None
) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.find_pod(name_prefix=...)``."""
    _warn("get_pod", "kube.find_pod(name_prefix=...)")
    kube = KubectlSession(sesh, namespace)
    pod_name = kube.find_pod(name_prefix=deployment)
    if output:
        return kube.get("pod", name=pod_name, output_format=output)
    return ExecResult(exit_code=0, stdout=pod_name)


@block(name="Exec in Pod", category="k8s", action="exec")
def exec(ctx, sesh: SSHSession, *, namespace: str, deployment: str, cmd: str) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.exec(target=..., cmd=...)``."""
    _warn("exec", "kube.exec(target=..., cmd=...)")
    return KubectlSession(sesh, namespace).exec(target=f"deploy/{deployment}", cmd=cmd)


@block(name="Copy from Pod", category="k8s", action="cp")
def cp(
    ctx, sesh: SSHSession, *, namespace: str, deployment: str, src: str, dest: str
) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.find_pod() + kube.cp(pod=...)``."""
    _warn("cp", "kube.find_pod() + kube.cp(pod=...)")
    kube = KubectlSession(sesh, namespace)
    return kube.cp(pod=kube.find_pod(name_prefix=deployment), src=src, dest=dest)


@block(name="Apply", category="k8s", action="apply")
def apply(ctx, sesh: SSHSession, *, file: str) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.apply(file=...)``."""
    _warn("apply", "kube.apply(file=...)")
    return KubectlSession(sesh, "default").apply(file=file)


@block(name="Rollout Status", category="k8s", action="rollout_status")
def rollout_status(
    ctx, sesh: SSHSession, *, namespace: str, deployment: str, timeout: int = 120
) -> ExecResult:
    """.. deprecated:: 0.2.0 Use ``kube.rollout_status(resource=...)``."""
    _warn("rollout_status", "kube.rollout_status(resource=...)")
    return KubectlSession(sesh, namespace).rollout_status(
        resource=f"deployment/{deployment}", timeout=timeout
    )
