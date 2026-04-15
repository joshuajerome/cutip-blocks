"""Type stubs for cutip_blocks.container — container lifecycle via Docker/Podman API."""

from typing import Any

class ContainerExecResult:
    """Result of a container exec operation."""

    exit_code: int
    stdout: str
    stderr: str

class ContainerRuntime:
    """Connection to Docker/Podman daemon. Created via container.connect()."""

    def build(
        self,
        *,
        context: str,
        dockerfile: str,
        tag: str,
        build_args: dict[str, str] | None = None,
        network_mode: str | None = None,
    ) -> str:
        """Build an image from a Dockerfile.

        Streams build output to stderr.

        Args:
            context: Path to the build context directory.
            dockerfile: Dockerfile filename (relative to context).
            tag: Image tag (e.g. "my-app:latest").
            build_args: Build-time arguments.
            network_mode: Network mode for build (e.g. "host").

        Returns:
            Image ID.
        """
        ...

    def create(
        self,
        *,
        name: str,
        image: str,
        network_mode: str | None = None,
        privileged: bool = False,
        hostname: str | None = None,
        workdir: str | None = None,
        command: str | None = None,
        environment: dict[str, str] | None = None,
        mounts: list[dict[str, Any]] | None = None,
        labels: dict[str, str] | None = None,
        ports: dict[str, str] | None = None,
        restart_policy: str | None = None,
    ) -> str:
        """Create a container from an image.

        Args:
            name: Container name.
            image: Image name or ID.
            network_mode: Network mode ("host", "bridge", or network name).
            privileged: Run in privileged mode.
            hostname: Container hostname.
            workdir: Working directory inside container.
            command: Command to run (passed to /bin/sh -c).
            environment: Environment variables.
            mounts: List of mount dicts with source, target, read_only keys.
            labels: Container labels.
            ports: Port mappings ("container_port/proto" -> "host_port").
            restart_policy: Restart policy ("always", "on-failure", "unless-stopped").

        Returns:
            Container ID.
        """
        ...

    def start(self, name: str) -> None:
        """Start a container by name."""
        ...

    def stop(self, name: str, timeout: int = 10) -> None:
        """Stop a running container."""
        ...

    def remove(self, name: str, force: bool = False) -> None:
        """Remove a container."""
        ...

    def exec(self, name: str, cmd: str) -> ContainerExecResult:
        """Execute a command inside a running container.

        Args:
            name: Container name.
            cmd: Command string (passed to bash -c).

        Returns:
            ContainerExecResult with exit_code, stdout, stderr.
        """
        ...

    def pull(self, image: str, tag: str = "latest") -> None:
        """Pull an image from a registry."""
        ...

    def exists(self, name: str) -> bool:
        """Check if a container exists by name."""
        ...

def connect(socket: str | None = None) -> ContainerRuntime:
    """Connect to Docker/Podman daemon.

    Args:
        socket: Optional path to Docker/Podman socket. Auto-detects if omitted.

    Returns:
        ContainerRuntime with build/create/start/stop/remove/exec/pull/exists methods.
    """
    ...
