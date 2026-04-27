"""Type stubs for rsty._core (Rust-backed PyO3 module).

Auto-generated from src/*.rs PyO3 signatures. Hand-edited if Rust signatures change.
"""
from __future__ import annotations

from typing import Any

# ── errors ────────────────────────────────────────────────────────────────

class CutipBlocksError(Exception):
    """Base error for all cutip-blocks operations."""

class ConnectionError(CutipBlocksError):
    """Failed to establish or maintain a connection (SSH, Docker, HTTP). Retryable."""

class TimeoutError(CutipBlocksError):
    """Operation exceeded its time limit. Retryable."""

class AuthError(CutipBlocksError):
    """Authentication or authorization failed. Not retryable."""

class CommandFailed(CutipBlocksError):
    """Command executed but returned a non-zero exit code."""

class ValidationError(CutipBlocksError):
    """Input validation failed (bad path, missing env var, invalid IP). Not retryable."""

# ── ssh ────────────────────────────────────────────────────────────────────

class ExecResult:
    """Result of a remote command execution."""
    exit_code: int
    stdout: str
    stderr: str

class SSHSession:
    """Persistent SSH session. Created via `ssh_connect()`."""
    def exec(self, cmd: str) -> ExecResult:
        """Execute a remote command."""
        ...
    def probe(self) -> ExecResult:
        """Run `whoami` to verify SSH connectivity."""
        ...
    def close(self) -> None:
        """Close the SSH connection."""
        ...
    def shell(self) -> ShellSession:
        """Open an interactive PTY shell session."""
        ...
    def upload(self, local_path: str, remote_path: str) -> None:
        """Upload a local file to the remote host via SFTP. Overwrites if exists."""
        ...
    def download(self, remote_path: str, local_path: str) -> None:
        """Download a remote file to the local path via SFTP. Overwrites if exists."""
        ...

class ShellSession:
    """Interactive PTY shell session with expect/send pattern matching."""
    def expect(self, pattern: str, timeout: int = 30) -> str:
        """Wait until a regex pattern appears in the output."""
        ...
    def send(self, text: str) -> None:
        """Send text to the shell's stdin (no newline appended)."""
        ...
    def send_line(self, text: str) -> None:
        """Send text followed by a newline."""
        ...
    def read_until(self, pattern: str, timeout: int = 30) -> str:
        """Read all output until a regex pattern is found."""
        ...
    def read_line(self, timeout: int = 10) -> str:
        """Read the next line of output."""
        ...
    def peek(self) -> str:
        """Return any data currently in the buffer without waiting."""
        ...
    def close(self) -> None:
        """Close the shell session."""
        ...

def ssh_connect(*, host: str, username: str, password: str, port: int = 22) -> SSHSession:
    """Open an SSH connection."""
    ...

# ── kubectl ───────────────────────────────────────────────────────────────

class KubectlSession:
    """Bound kubectl session over an SSH connection."""
    def get(
        self,
        resource: str,
        *,
        name: str,
        namespace: str | None = None,
        output_format: str = "name",
    ) -> ExecResult:
        """Run `kubectl get <resource> -n <ns> <name> -o <format>`."""
        ...
    def get_secret_value(
        self,
        *,
        secret: str,
        key: str,
        namespace: str | None = None,
    ) -> str:
        """Decode a base64-encoded field from a Kubernetes secret."""
        ...
    def find_pod(
        self,
        *,
        name_prefix: str,
        namespace: str | None = None,
    ) -> str:
        """Find a pod by name prefix. Prefers Running pods."""
        ...
    def exec(
        self,
        *,
        target: str,
        cmd: str,
        namespace: str | None = None,
    ) -> ExecResult:
        """Run `kubectl exec <target> -- bash -c '<cmd>'`."""
        ...
    def cat_file(
        self,
        *,
        target: str,
        path: str,
        namespace: str | None = None,
    ) -> str:
        """Read a file from inside a pod (no tar needed)."""
        ...
    def cp(
        self,
        *,
        pod: str,
        src: str,
        dest: str,
        namespace: str | None = None,
    ) -> ExecResult:
        """Run `kubectl cp <ns>/<pod>:<src> <dest>`."""
        ...
    def apply(self, *, file: str) -> ExecResult:
        """Run `kubectl apply -f <file>`."""
        ...
    def rollout_status(
        self,
        *,
        resource: str,
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
        source_file: str,
        dest_dir: str,
        replacements: dict[str, str],
        chmod: str = "777",
        namespace: str | None = None,
    ) -> str:
        """Copy a file from a pod, apply sed replacements, chmod. Re-run safe."""
        ...

def kubectl_connect(sesh: SSHSession, *, namespace: str) -> KubectlSession:
    """Create a bound kubectl session over an existing SSH connection."""
    ...

# ── container ─────────────────────────────────────────────────────────────

class ContainerExecResult:
    """Result of a container exec operation."""
    exit_code: int
    stdout: str
    stderr: str

class ContainerRuntime:
    """A connection to the Docker/Podman daemon."""
    def build(
        self,
        *,
        context: str,
        dockerfile: str,
        tag: str,
        build_args: dict[str, str] | None = None,
        network_mode: str | None = None,
        timeout: int = 300,
    ) -> str:
        """Build an image from a Dockerfile. Returns the image ID."""
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
        """Create a container from an image. Returns the container ID."""
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
        """Execute a command inside a running container."""
        ...
    def pull(self, image: str, tag: str = "latest") -> None:
        """Pull an image from a registry."""
        ...
    def exists(self, name: str) -> bool:
        """Check if a container exists by name."""
        ...

def container_connect(socket: str | None = None) -> ContainerRuntime:
    """Connect to the Docker/Podman daemon."""
    ...

# ── file ──────────────────────────────────────────────────────────────────

def copy(src: str, dest: str) -> str:
    """Copy a single file."""
    ...

def copy_tree(src: str, dest: str, clean: bool = False) -> str:
    """Copy a directory recursively."""
    ...

def read_yaml(path: str) -> Any:
    """Read and parse a YAML file, returning a Python dict."""
    ...

def write_yaml(path: str, data: dict[str, Any]) -> str:
    """Write a Python dict to a YAML file."""
    ...

def read_json(path: str) -> Any:
    """Read and parse a JSON file, returning a Python dict."""
    ...

def write_json(path: str, data: dict[str, Any]) -> str:
    """Write a Python dict to a JSON file."""
    ...

def replace(path: str, old: str, new: str) -> str:
    """Replace all occurrences of `old` with `new` in a file."""
    ...

def mkdir(path: str) -> str:
    """Create a directory (and parents) if it doesn't exist."""
    ...

def is_empty(value: str | None) -> bool:
    """Check if a string/bytes value is empty or whitespace-only."""
    ...

# ── http ──────────────────────────────────────────────────────────────────

class HttpResponse:
    """Result of an HTTP request."""
    status_code: int
    text: str
    ok: bool
    @property
    def bytes(self) -> bytes:
        """Raw response body. Use this for binary downloads."""
        ...
    def json(self) -> Any:
        """Parse the response body as JSON, returning a Python dict."""
        ...

def get(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP GET request."""
    ...

def post(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP POST request with JSON body."""
    ...

def put(
    url: str,
    *,
    json: dict[str, Any] | None = None,
    body: str | None = None,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP PUT request with JSON body."""
    ...

def delete(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    verify_tls: bool = True,
    timeout_s: int = 30,
) -> HttpResponse:
    """Send an HTTP DELETE request."""
    ...

# ── network ───────────────────────────────────────────────────────────────

def create(
    runtime: ContainerRuntime,
    name: str,
    *,
    driver: str = "bridge",
    subnet: str | None = None,
    gateway: str | None = None,
) -> None:
    """Create a Docker/Podman network."""
    ...

def exists(runtime: ContainerRuntime, name: str) -> bool:
    """Check if a network exists."""
    ...

# Note: `network::remove` is registered first in src/lib.rs, then overwritten
# by `pkg::remove`. The single `_core.remove` symbol resolves to pkg::remove
# at runtime. The network wrapper's call `_remove(runtime, name)` is therefore
# a latent bug — see rsty/network.py. The stub below reflects the actual
# runtime symbol.

# ── notify ────────────────────────────────────────────────────────────────

def send(title: str, message: str, *, sound: bool = True) -> None:
    """Send an OS-native toast notification."""
    ...

# ── config ────────────────────────────────────────────────────────────────

def render_template(template: str, vars: dict[str, str]) -> str:
    """Replace `{{ key }}` placeholders in a template string with values from a vars dict."""
    ...

def substitute_vars(
    text: str,
    vars: dict[str, str],
    secrets: dict[str, str] | None = None,
) -> str:
    """Replace `{{ vars.key }}` and `{{ secrets.key }}` placeholders in text."""
    ...

# ── crypto ────────────────────────────────────────────────────────────────

def sign_rsa_sha256(key_path: str, data: str) -> str:
    """Sign data with an RSA private key using PKCS1v15 + SHA-256."""
    ...

def base64_encode(data: str) -> str:
    """Base64-encode a string."""
    ...

def base64_decode(data: str) -> str:
    """Base64-decode a string."""
    ...

# ── shell ─────────────────────────────────────────────────────────────────

def run(
    cmd: str,
    *,
    cwd: str | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
    stream: bool = False,
) -> ExecResult:
    """Execute a shell command on the local host."""
    ...

# ── service ───────────────────────────────────────────────────────────────

def poll_until_ready(
    url: str,
    *,
    retries: int = 30,
    interval_s: int = 1,
    verify_tls: bool = True,
) -> None:
    """Poll an HTTP endpoint until it returns a success status code."""
    ...

def wait_for_exit(
    runtime: ContainerRuntime,
    container_name: str,
    *,
    timeout_s: int = 120,
) -> int:
    """Wait for a container to exit (via Docker API)."""
    ...

# ── svc ───────────────────────────────────────────────────────────────────

def start(service: str) -> None:
    """Start a systemd service."""
    ...

def stop(service: str) -> None:
    """Stop a systemd service."""
    ...

def restart(service: str) -> None:
    """Restart a systemd service."""
    ...

def enable(service: str) -> None:
    """Enable a systemd service (start on boot)."""
    ...

def disable(service: str) -> None:
    """Disable a systemd service."""
    ...

def is_active(service: str) -> bool:
    """Check if a systemd service is active (running)."""
    ...

def status(service: str) -> ExecResult:
    """Get the status of a systemd service."""
    ...

# ── template ──────────────────────────────────────────────────────────────

def render(
    src: str,
    vars: dict[str, str],
    dest: str | None = None,
) -> str:
    """Render a template file, replacing {{ key }} placeholders with values."""
    ...

def render_string(text: str, vars: dict[str, str]) -> str:
    """Render a template string (not a file), replacing {{ key }} placeholders."""
    ...

def check(src: str, vars: dict[str, str]) -> list[str]:
    """Check that all {{ key }} placeholders in a template have values in vars."""
    ...

# ── validate ──────────────────────────────────────────────────────────────

def path_exists(path: str) -> bool:
    """Check if a file or directory exists at the given path."""
    ...

def env_var_set(name: str) -> bool:
    """Check if an environment variable is set and non-empty."""
    ...

def ip_valid(addr: str) -> bool:
    """Check if a string is a valid IPv4 or IPv6 address."""
    ...

# ── pkg ───────────────────────────────────────────────────────────────────

def install(packages: list[str]) -> None:
    """Install system packages. Auto-detects apt/dnf/yum/apk."""
    ...

def remove(packages: list[str]) -> None:
    """Remove system packages. Auto-detects apt/dnf/yum/apk."""
    ...

def update() -> None:
    """Update the package index. Auto-detects apt/dnf/yum/apk."""
    ...
