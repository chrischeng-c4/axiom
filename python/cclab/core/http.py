"""Shared HTTP types for data-bridge ecosystem.

This module provides base classes for HTTP request and response types
that are shared between the API server and HTTP client modules.
"""

from abc import ABC, abstractmethod
from enum import Enum
from typing import Any, Dict, Optional


class HttpMethod(str, Enum):
    """HTTP request methods."""
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    PATCH = "PATCH"
    DELETE = "DELETE"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"

    def __str__(self) -> str:
        return self.value


class HttpStatus:
    """HTTP status code wrapper with helper methods."""

    # Common status codes as class constants
    OK: "HttpStatus"
    CREATED: "HttpStatus"
    NO_CONTENT: "HttpStatus"
    BAD_REQUEST: "HttpStatus"
    UNAUTHORIZED: "HttpStatus"
    FORBIDDEN: "HttpStatus"
    NOT_FOUND: "HttpStatus"
    INTERNAL_SERVER_ERROR: "HttpStatus"

    def __init__(self, code: int):
        self.code = code

    def is_success(self) -> bool:
        """Returns True if this is a success status (2xx)."""
        return 200 <= self.code < 300

    def is_client_error(self) -> bool:
        """Returns True if this is a client error status (4xx)."""
        return 400 <= self.code < 500

    def is_server_error(self) -> bool:
        """Returns True if this is a server error status (5xx)."""
        return 500 <= self.code < 600

    def is_redirect(self) -> bool:
        """Returns True if this is a redirect status (3xx)."""
        return 300 <= self.code < 400


# Set class constants after class is defined
HttpStatus.OK = HttpStatus(200)
HttpStatus.CREATED = HttpStatus(201)
HttpStatus.NO_CONTENT = HttpStatus(204)
HttpStatus.BAD_REQUEST = HttpStatus(400)
HttpStatus.UNAUTHORIZED = HttpStatus(401)
HttpStatus.FORBIDDEN = HttpStatus(403)
HttpStatus.NOT_FOUND = HttpStatus(404)
HttpStatus.INTERNAL_SERVER_ERROR = HttpStatus(500)


class BaseResponse(ABC):
    """Base class for HTTP response types.

    This provides a consistent interface for response types across
    the API server and HTTP client modules.

    Subclasses must implement:
        - body_bytes(): Return the response body as bytes

    Example:
        >>> class MyResponse(BaseResponse):
        ...     _body: bytes = b""
        ...     def body_bytes(self) -> bytes:
        ...         return self._body
        >>> resp = MyResponse(status_code=200)
        >>> resp.is_success()
        True
    """

    def __init__(
        self,
        status_code: int,
        headers: Optional[Dict[str, str]] = None,
    ):
        self.status_code = status_code
        self.headers = headers if headers is not None else {}

    def is_success(self) -> bool:
        """Returns True if this is a success response (2xx)."""
        return 200 <= self.status_code < 300

    def is_client_error(self) -> bool:
        """Returns True if this is a client error response (4xx)."""
        return 400 <= self.status_code < 500

    def is_server_error(self) -> bool:
        """Returns True if this is a server error response (5xx)."""
        return 500 <= self.status_code < 600

    def is_redirect(self) -> bool:
        """Returns True if this is a redirect response (3xx)."""
        return 300 <= self.status_code < 400

    def header(self, name: str) -> Optional[str]:
        """Get a header value by name (case-insensitive).

        Args:
            name: The header name to look up

        Returns:
            The header value if found, None otherwise
        """
        name_lower = name.lower()
        for key, value in self.headers.items():
            if key.lower() == name_lower:
                return value
        return None

    @property
    def content_type(self) -> Optional[str]:
        """Returns the Content-Type header value."""
        return self.header("content-type")

    @property
    def content_length(self) -> Optional[int]:
        """Returns the Content-Length header value as an integer."""
        value = self.header("content-length")
        if value is not None:
            try:
                return int(value)
            except ValueError:
                return None
        return None

    @property
    def status(self) -> HttpStatus:
        """Returns the status as an HttpStatus object."""
        return HttpStatus(self.status_code)

    @abstractmethod
    def body_bytes(self) -> bytes:
        """Return the response body as bytes.

        Subclasses must implement this method.

        Returns:
            The response body as bytes
        """
        ...


class BaseRequest(ABC):
    """Base class for HTTP request types.

    This provides a consistent interface for request types across
    the API server and HTTP client modules.

    Example:
        >>> class MyRequest(BaseRequest):
        ...     _body: bytes = b""
        ...     def body_bytes(self) -> Optional[bytes]:
        ...         return self._body if self._body else None
    """

    def __init__(
        self,
        method: str,
        url: str,
        headers: Optional[Dict[str, str]] = None,
    ):
        self.method = method
        self.url = url
        self.headers = headers if headers is not None else {}

    def header(self, name: str) -> Optional[str]:
        """Get a header value by name (case-insensitive).

        Args:
            name: The header name to look up

        Returns:
            The header value if found, None otherwise
        """
        name_lower = name.lower()
        for key, value in self.headers.items():
            if key.lower() == name_lower:
                return value
        return None

    @property
    def content_type(self) -> Optional[str]:
        """Returns the Content-Type header value."""
        return self.header("content-type")

    @abstractmethod
    def body_bytes(self) -> Optional[bytes]:
        """Return the request body as bytes.

        Subclasses must implement this method.

        Returns:
            The request body as bytes, or None if no body
        """
        ...
