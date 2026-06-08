"""
High-performance async HTTP client with Rust backend.

This module provides a fast HTTP client that bypasses the GIL for maximum
throughput. It wraps the data-bridge-http Rust crate.

Example:
    ```python
    from cclab.http import HttpClient

    # Create client
    client = HttpClient(
        base_url="https://api.example.com",
        timeout=30.0,
    )

    # Make async requests
    response = await client.get("/users")
    print(response.status_code, response.latency_ms)

    # Parse JSON response
    data = response.json()
    ```

Classes:
    HttpClient: High-performance async HTTP client with connection pooling.
        Args:
            base_url: Base URL for all requests (optional)
            timeout: Total request timeout in seconds (default: 30.0)
            connect_timeout: Connection timeout in seconds (default: 10.0)
            pool_max_idle_per_host: Max idle connections per host (default: 10)
            follow_redirects: Whether to follow redirects (default: True)
            max_redirects: Maximum redirects to follow (default: 10)
            user_agent: Custom User-Agent header (optional)
            danger_accept_invalid_certs: Accept invalid TLS certs (default: False)

        Methods:
            get(path, headers=None, params=None, timeout=None) -> HttpResponse
            post(path, headers=None, params=None, json=None, form=None, timeout=None) -> HttpResponse
            put(path, headers=None, params=None, json=None, form=None, timeout=None) -> HttpResponse
            patch(path, headers=None, params=None, json=None, form=None, timeout=None) -> HttpResponse
            delete(path, headers=None, params=None, timeout=None) -> HttpResponse
            head(path, headers=None, params=None, timeout=None) -> HttpResponse
            options(path, headers=None, params=None, timeout=None) -> HttpResponse

    HttpResponse: HTTP response with status, headers, body, and latency.
        Attributes:
            status_code: HTTP status code (e.g., 200, 404)
            latency_ms: Request latency in milliseconds
            url: Final URL after redirects

        Methods:
            is_success() -> bool: Check if 2xx status
            is_client_error() -> bool: Check if 4xx status
            is_server_error() -> bool: Check if 5xx status
            text() -> str: Get body as UTF-8 text
            json() -> Any: Parse body as JSON
            bytes() -> bytes: Get body as bytes
            header(name) -> str | None: Get header (case-insensitive)
            content_type() -> str | None: Get Content-Type header
"""

from __future__ import annotations

# Import from the Rust extension (cclab.abi3.so in parent directory)
from ..cclab import http as _http_rust
HttpClient = _http_rust.HttpClient
HttpResponse = _http_rust.HttpResponse

__all__ = [
    "HttpClient",
    "HttpResponse",
]
