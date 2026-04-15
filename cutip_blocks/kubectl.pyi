"""Type stubs for cutip_blocks.kubectl — Kubernetes operations over SSH."""

from cutip_blocks.ssh import ExecResult, SSHSession

class KubectlSession:
    """Bound kubectl session over an SSH connection. Created via kubectl.connect()."""

    def get(
        self,
        resource: str,
        *,
        name: str,
        namespace: str | None = None,
        output_format: str = "name",
    ) -> ExecResult:
        """Run kubectl get.

        Args:
            resource: Resource type (e.g. "deployment", "pod").
            name: Resource name.
            namespace: Override namespace (default: session namespace).
            output_format: Output format (default: "name").
        """
        ...

    def get_secret_value(
        self,
        *,
        secret: str,
        key: str,
        namespace: str | None = None,
    ) -> str:
        """Decode a base64-encoded field from a Kubernetes secret.

        Args:
            secret: Secret name.
            key: Key within the secret.
            namespace: Override namespace.

        Returns:
            Decoded secret value.
        """
        ...

    def find_pod(
        self,
        name_prefix: str,
        *,
        namespace: str | None = None,
    ) -> str:
        """Find a pod by name prefix, preferring Running status.

        Args:
            name_prefix: Pod name prefix.
            namespace: Override namespace.

        Returns:
            Full pod name.
        """
        ...

    def exec(
        self,
        target: str,
        cmd: str,
        *,
        namespace: str | None = None,
    ) -> ExecResult:
        """Execute a command inside a pod.

        Args:
            target: Pod or deployment target (e.g. "deploy/web").
            cmd: Command string.
            namespace: Override namespace.
        """
        ...

    def cat_file(
        self,
        target: str,
        path: str,
        *,
        namespace: str | None = None,
    ) -> str:
        """Read a file from a pod.

        Args:
            target: Pod target.
            path: File path inside pod.
            namespace: Override namespace.

        Returns:
            File contents.
        """
        ...

    def cp(
        self,
        pod: str,
        src: str,
        dest: str,
        *,
        namespace: str | None = None,
    ) -> ExecResult:
        """Copy files to/from a pod via kubectl cp."""
        ...

    def apply(
        self,
        file: str,
        *,
        namespace: str | None = None,
    ) -> ExecResult:
        """Apply a YAML file via kubectl apply."""
        ...

    def rollout_status(
        self,
        resource: str,
        *,
        namespace: str | None = None,
        timeout: int = 120,
    ) -> ExecResult:
        """Wait for a rollout to complete."""
        ...

    def patch_deployment(
        self,
        *,
        deployment: str,
        volume_name: str,
        host_path: str,
        mount_path: str,
        namespace: str | None = None,
        rollout_timeout: int = 120,
    ) -> ExecResult:
        """Patch a deployment to add a hostPath volume + volumeMount."""
        ...

    def patch_file_from_pod(
        self,
        *,
        deployment: str,
        pod_file_path: str,
        host_file_path: str,
        search: str,
        replace: str,
        namespace: str | None = None,
        chmod: str | None = None,
        rollout_timeout: int = 120,
    ) -> ExecResult:
        """Copy a file from a pod, patch it, mount it back via hostPath."""
        ...

def connect(sesh: SSHSession, *, namespace: str) -> KubectlSession:
    """Create a bound kubectl session over an existing SSH connection.

    Args:
        sesh: An active SSHSession.
        namespace: Default Kubernetes namespace for all commands.

    Returns:
        KubectlSession bound to the SSH connection and namespace.
    """
    ...
