"""Type stubs for rsty.errors — structured error types."""

class CutipBlocksError(Exception):
    """Base error for all cutip-blocks operations."""
    ...

class ConnectionError(CutipBlocksError):
    """Failed to establish or maintain a connection. Retryable."""
    ...

class TimeoutError(CutipBlocksError):
    """Operation exceeded its time limit. Retryable."""
    ...

class AuthError(CutipBlocksError):
    """Authentication or authorization failed. Not retryable."""
    ...

class CommandFailed(CutipBlocksError):
    """Command executed but returned a non-zero exit code."""
    ...

class ValidationError(CutipBlocksError):
    """Input validation failed. Not retryable."""
    ...
