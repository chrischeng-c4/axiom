from typing import Any

__all__ = ["HttpClient", "HttpResponse"]

class HttpClient:
    """Python HttpClient class"""
    def __init__(self, base_url: str | None = None, timeout: float = 30.0, connect_timeout: float = 10.0, pool_max_idle_per_host: int = 10, follow_redirects: bool = True, max_redirects: int = 10, user_agent: str | None = None, danger_accept_invalid_certs: bool = False) -> None:
        ...
    @property
    def base_url(self) -> str | None:
        """Get the base URL"""
        ...
    def get(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a GET request"""
        ...
    def post(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, json: Any | None = None, form: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a POST request"""
        ...
    def put(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, json: Any | None = None, form: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a PUT request"""
        ...
    def patch(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, json: Any | None = None, form: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a PATCH request"""
        ...
    def delete(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a DELETE request"""
        ...
    def head(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send a HEAD request"""
        ...
    def options(self, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Send an OPTIONS request"""
        ...
    def request(self, method: str, path: str, headers: dict[Any, Any] | None = None, params: dict[Any, Any] | None = None, json: Any | None = None, form: dict[Any, Any] | None = None, timeout: float | None = None) -> Any:
        """Generic request method"""
        ...

class HttpResponse:
    """Python HttpResponse class"""
    status_code: int
    latency_ms: int
    url: str
    @property
    def headers(self) -> Any:
        """Get response headers as a dict"""
        ...
    def is_success(self) -> bool:
        """Check if response is successful (2xx)"""
        ...
    def is_client_error(self) -> bool:
        """Check if response is client error (4xx)"""
        ...
    def is_server_error(self) -> bool:
        """Check if response is server error (5xx)"""
        ...
    def text(self) -> str:
        """Get body as text (UTF-8)"""
        ...
    def json(self) -> Any:
        """Get body as JSON (using pythonize for efficient conversion)"""
        ...
    def bytes(self) -> Any:
        """Get body as bytes"""
        ...
    def content_length(self) -> int:
        """Get content length"""
        ...
    def header(self, name: str) -> str | None:
        """Get a header value (case-insensitive)"""
        ...
    def content_type(self) -> str | None:
        """Get content type"""
        ...
    def __repr__(self) -> str:
        ...

