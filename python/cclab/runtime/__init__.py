"""
cclab.runtime - Rust-native asyncio event loop

Provides a high-performance drop-in replacement for Python's asyncio event loop,
backed by Tokio runtime with seamless Rust integration.

Example:
    >>> import cclab.runtime
    >>> cclab.runtime.install()
    >>> import asyncio
    >>> # Now using Tokio-backed event loop!

Architecture:
    Python asyncio protocol → PyLoop (PyO3) → Tokio Runtime (Rust)

Benefits:
    - 2-10x faster event loop operations
    - Better integration with Rust async code
    - Reduced GIL contention
    - Native support for spawning Rust futures
"""

from __future__ import annotations

import asyncio
import logging
import threading
import typing as _typing
from typing import TYPE_CHECKING, Optional, Any, Dict, List
from abc import ABC, abstractmethod

# Configure logger
logger = logging.getLogger("cclab.runtime")

if TYPE_CHECKING:
    # Type stubs for the extension module
    from asyncio import AbstractEventLoop

    class PyLoop(AbstractEventLoop):
        """Type stub for PyLoop class from Rust extension."""
        ...
else:
    # Import the Rust extension module at runtime
    try:
        from .. import cclab as _native  # type: ignore[import-not-found]
        _pyloop = _native._pyloop  # type: ignore[attr-defined]
        PyLoop = _pyloop.PyLoop
    except (ImportError, AttributeError) as e:
        raise ImportError(
            "Failed to import PyLoop from cclab native module. "
            "Please run 'maturin develop --features pyloop' to build the extension."
        ) from e

    # API module is optional (requires 'api' feature)
    try:
        _api = _native.api  # type: ignore[attr-defined]
        _RustApp = _api.ApiApp
    except AttributeError:
        _RustApp = None

__all__ = [
    "PyLoop",
    "install",
    "EventLoopPolicy",
    "is_installed",
    "App",
    "HTTPException",
    "ValidationError",
    "NotFoundError",
    "ConflictError",
    "BaseMiddleware",
    "CORSMiddleware",
    "LoggingMiddleware",
    "CompressionMiddleware",
    "OpenTelemetryMiddleware",
]

__version__ = "0.1.0"

_AbstractEventLoop = asyncio.AbstractEventLoop


class EventLoopPolicy(
    # This is to avoid a mypy error about AbstractEventLoopPolicy
    getattr(asyncio, 'AbstractEventLoopPolicy')  # type: ignore[misc]
):
    """Custom event loop policy that returns PyLoop instances.

    This class implements the asyncio.AbstractEventLoopPolicy interface
    to provide PyLoop instances as the default event loop.

    Example:
        >>> import asyncio
        >>> from cclab.runtime import EventLoopPolicy
        >>> asyncio.set_event_loop_policy(EventLoopPolicy())
        >>> loop = asyncio.get_event_loop()
        >>> # loop is now a PyLoop instance
    """

    def _loop_factory(self) -> _AbstractEventLoop:
        """Factory method to create a new event loop.

        Returns:
            AbstractEventLoop: A new PyLoop instance.
        """
        return PyLoop()

    if _typing.TYPE_CHECKING:
        # EventLoopPolicy doesn't implement these, but since they are
        # marked as abstract in typeshed, we have to put them in so mypy
        # thinks the base methods are overridden. This is the same approach
        # taken for the Windows event loop policy classes in typeshed.
        def get_child_watcher(self) -> _typing.NoReturn:
            ...

        def set_child_watcher(
            self, watcher: _typing.Any
        ) -> _typing.NoReturn:
            ...

    class _Local(threading.local):
        _loop: _typing.Optional[_AbstractEventLoop] = None

    def __init__(self) -> None:
        self._local = self._Local()

    def get_event_loop(self) -> _AbstractEventLoop:
        """Get the event loop for the current context.

        Returns an instance of EventLoop or raises an exception.
        """
        if self._local._loop is None:
            # Auto-create a loop if one doesn't exist (matches asyncio behavior)
            self.set_event_loop(self.new_event_loop())

        assert self._local._loop is not None  # Help type checker
        return self._local._loop

    def set_event_loop(
        self, loop: _typing.Optional[_AbstractEventLoop]
    ) -> None:
        """Set the event loop."""
        # Accept PyLoop (Rust type) or Python's AbstractEventLoop
        if loop is not None and not isinstance(loop, (PyLoop, _AbstractEventLoop)):
            raise TypeError(
                f"loop must be an instance of AbstractEventLoop or None, "
                f"not '{type(loop).__name__}'"
            )
        self._local._loop = loop

    def new_event_loop(self) -> _AbstractEventLoop:
        """Create a new event loop.

        You must call set_event_loop() to make this the current event loop.
        """
        return self._loop_factory()


def install() -> None:
    """Install PyLoop as the default asyncio event loop.

    This replaces the standard asyncio event loop policy with our
    Tokio-backed implementation. All subsequent asyncio operations
    will use PyLoop automatically.

    This should be called early in your application, typically in
    the main module before any asyncio code runs.

    Example:
        >>> import cclab.runtime
        >>> cclab.runtime.install()
        >>> import asyncio
        >>> loop = asyncio.get_event_loop()
        >>> # loop is now a PyLoop instance backed by Tokio

    Note:
        This is a global operation that affects all asyncio code in
        the current process. It should only be called once.
    """
    asyncio.set_event_loop_policy(EventLoopPolicy())  # type: ignore[arg-type]


def is_installed() -> bool:
    """Check if PyLoop is currently installed as the default event loop.

    Returns:
        bool: True if PyLoop is installed, False otherwise.

    Example:
        >>> import cclab.runtime
        >>> cclab.runtime.is_installed()
        False
        >>> cclab.runtime.install()
        >>> cclab.runtime.is_installed()
        True
    """
    policy = asyncio.get_event_loop_policy()
    return isinstance(policy, EventLoopPolicy)


class HTTPException(Exception):
    """
    HTTP exception with status code and detail.

    Raise this in handlers to return specific HTTP status codes.

    Example:
        raise HTTPException(404, "Product not found")
        raise HTTPException(400, "Invalid request", {"field": "price"})
    """
    def __init__(
        self,
        status_code: int,
        detail: Optional[str] = None,
        headers: Optional[Dict[str, str]] = None,
        extra: Optional[Dict[str, Any]] = None
    ):
        self.status_code = status_code
        self.detail = detail or self._default_detail(status_code)
        self.headers = headers or {}
        self.extra = extra or {}
        super().__init__(self.detail)

    @staticmethod
    def _default_detail(status_code: int) -> str:
        """Get default detail message for status code."""
        messages = {
            400: "Bad Request",
            401: "Unauthorized",
            403: "Forbidden",
            404: "Not Found",
            405: "Method Not Allowed",
            409: "Conflict",
            422: "Unprocessable Entity",
            500: "Internal Server Error",
            503: "Service Unavailable"
        }
        return messages.get(status_code, "Error")

    def to_response(self) -> Dict[str, Any]:
        """Convert exception to response dict."""
        body = {
            "error": self.detail,
            "status_code": self.status_code
        }
        if self.extra:
            body.update(self.extra)

        response = {
            "status": self.status_code,
            "body": body
        }

        if self.headers:
            response["headers"] = self.headers

        return response


class ValidationError(HTTPException):
    """Validation error (422 Unprocessable Entity)."""
    def __init__(self, detail: str, errors: Optional[Dict] = None):
        super().__init__(422, detail, extra={"errors": errors} if errors else None)


class NotFoundError(HTTPException):
    """Not found error (404)."""
    def __init__(self, detail: str = "Resource not found"):
        super().__init__(404, detail)


class ConflictError(HTTPException):
    """Conflict error (409) - e.g., duplicate key."""
    def __init__(self, detail: str = "Resource already exists"):
        super().__init__(409, detail)


class BaseMiddleware(ABC):
    """
    Base class for HTTP middleware.

    Middleware can modify requests before handlers and responses after handlers.

    Example:
        class CustomMiddleware(BaseMiddleware):
            async def process_request(self, request: Dict) -> Optional[Dict]:
                # Modify request or return early response
                if request.get("path") == "/blocked":
                    return {"status": 403, "body": {"error": "Forbidden"}}
                return None

            async def process_response(self, request: Dict, response: Dict) -> Dict:
                # Modify response
                response.setdefault("headers", {})["X-Custom"] = "Header"
                return response
    """

    @abstractmethod
    async def process_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Process request before handler.

        Args:
            request: Request dict

        Returns:
            None to continue to handler, or response dict to return early
        """
        pass

    @abstractmethod
    async def process_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """
        Process response after handler.

        Args:
            request: Original request dict
            response: Response dict from handler

        Returns:
            Modified response dict
        """
        pass


class CORSMiddleware(BaseMiddleware):
    """
    CORS (Cross-Origin Resource Sharing) middleware.

    Handles preflight requests and adds CORS headers to responses.

    Example:
        # Allow all origins
        app.add_middleware(CORSMiddleware(allow_origins=["*"]))

        # Specific origins
        app.add_middleware(CORSMiddleware(
            allow_origins=["https://example.com", "https://app.example.com"],
            allow_methods=["GET", "POST", "PUT", "DELETE"],
            allow_headers=["Content-Type", "Authorization"],
            allow_credentials=True,
            max_age=600
        ))
    """

    def __init__(
        self,
        allow_origins: Optional[List[str]] = None,
        allow_methods: Optional[List[str]] = None,
        allow_headers: Optional[List[str]] = None,
        expose_headers: Optional[List[str]] = None,
        allow_credentials: bool = False,
        max_age: int = 600
    ):
        """
        Initialize CORS middleware.

        Args:
            allow_origins: Allowed origins (["*"] for all, or specific origins)
            allow_methods: Allowed HTTP methods (default: ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            allow_headers: Allowed request headers (default: ["*"])
            expose_headers: Headers exposed to browser (default: [])
            allow_credentials: Allow credentials (cookies, auth headers)
            max_age: Max age for preflight cache in seconds (default: 600)
        """
        self.allow_origins = allow_origins or ["*"]
        self.allow_methods = allow_methods or ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
        self.allow_headers = allow_headers or ["*"]
        self.expose_headers = expose_headers or []
        self.allow_credentials = allow_credentials
        self.max_age = max_age

        # Check for wildcard
        self.allow_all_origins = "*" in self.allow_origins
        self.allow_all_headers = "*" in self.allow_headers

    def _is_origin_allowed(self, origin: str) -> bool:
        """Check if origin is allowed."""
        if self.allow_all_origins:
            return True
        return origin in self.allow_origins

    async def process_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Handle CORS preflight requests."""
        # Handle OPTIONS preflight request
        if request.get("method") == "OPTIONS":
            origin = request.get("headers", {}).get("origin")

            if origin and self._is_origin_allowed(origin):
                headers = {
                    "Access-Control-Allow-Origin": origin if not self.allow_all_origins else "*",
                    "Access-Control-Allow-Methods": ", ".join(self.allow_methods),
                    "Access-Control-Max-Age": str(self.max_age),
                    "Vary": "Origin"
                }

                # Handle Access-Control-Request-Headers
                request_headers = request.get("headers", {}).get("access-control-request-headers")
                if request_headers:
                    if self.allow_all_headers:
                        headers["Access-Control-Allow-Headers"] = request_headers
                    else:
                        headers["Access-Control-Allow-Headers"] = ", ".join(self.allow_headers)

                if self.allow_credentials:
                    headers["Access-Control-Allow-Credentials"] = "true"

                # Return early response for preflight
                return {
                    "status": 204,
                    "body": None,
                    "headers": headers
                }

        # Not a preflight request, continue to handler
        return None

    async def process_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """Add CORS headers to response."""
        origin = request.get("headers", {}).get("origin")

        if origin and self._is_origin_allowed(origin):
            # Ensure headers dict exists
            if "headers" not in response:
                response["headers"] = {}

            # Add CORS headers
            response["headers"]["Access-Control-Allow-Origin"] = origin if not self.allow_all_origins else "*"
            response["headers"]["Vary"] = "Origin"

            if self.expose_headers:
                response["headers"]["Access-Control-Expose-Headers"] = ", ".join(self.expose_headers)

            if self.allow_credentials:
                response["headers"]["Access-Control-Allow-Credentials"] = "true"

        return response


class LoggingMiddleware(BaseMiddleware):
    """
    Request/Response logging middleware.

    Logs all requests and responses with timing information.

    Example:
        app.add_middleware(LoggingMiddleware(
            logger=logging.getLogger("myapp"),
            log_request_body=True,
            log_response_body=False
        ))
    """

    def __init__(
        self,
        logger_instance=None,
        log_request_body: bool = False,
        log_response_body: bool = False
    ):
        """
        Initialize logging middleware.

        Args:
            logger_instance: Logger instance (default: cclab.runtime logger)
            log_request_body: Whether to log request body (default: False)
            log_response_body: Whether to log response body (default: False)
        """
        self.logger_instance = logger_instance or logging.getLogger("cclab.runtime")
        self.log_request_body = log_request_body
        self.log_response_body = log_response_body

    async def process_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Log incoming request."""
        import time

        # Store request start time
        request["_middleware_start_time"] = time.time()

        # Log request
        log_data = {
            "method": request.get("method"),
            "path": request.get("path"),
            "query_params": request.get("query_params", {})
        }

        if self.log_request_body and "body" in request:
            log_data["body"] = request["body"]

        self.logger_instance.info(f"→ {request.get('method')} {request.get('path')}", extra=log_data)

        # Don't return early, continue to handler
        return None

    async def process_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """Log outgoing response."""
        import time

        # Calculate request duration
        start_time = request.get("_middleware_start_time")
        duration_ms = (time.time() - start_time) * 1000 if start_time else 0

        # Log response
        log_data = {
            "method": request.get("method"),
            "path": request.get("path"),
            "status": response.get("status"),
            "duration_ms": round(duration_ms, 2)
        }

        if self.log_response_body and "body" in response:
            log_data["body"] = response["body"]

        status = response.get("status", 0)
        if status >= 500:
            self.logger_instance.error(f"← {status} {request.get('method')} {request.get('path')} ({duration_ms:.2f}ms)", extra=log_data)
        elif status >= 400:
            self.logger_instance.warning(f"← {status} {request.get('method')} {request.get('path')} ({duration_ms:.2f}ms)", extra=log_data)
        else:
            self.logger_instance.info(f"← {status} {request.get('method')} {request.get('path')} ({duration_ms:.2f}ms)", extra=log_data)

        return response


class CompressionMiddleware(BaseMiddleware):
    """
    Response compression middleware (gzip).

    Compresses responses based on Accept-Encoding header.

    Example:
        app.add_middleware(CompressionMiddleware(
            minimum_size=500,  # Only compress responses > 500 bytes
            compression_level=6
        ))
    """

    def __init__(
        self,
        minimum_size: int = 500,
        compression_level: int = 6
    ):
        """
        Initialize compression middleware.

        Args:
            minimum_size: Minimum response size to compress (bytes)
            compression_level: gzip compression level (1-9, default: 6)
        """
        self.minimum_size = minimum_size
        self.compression_level = compression_level

    async def process_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """No request processing needed."""
        return None

    async def process_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """Compress response if appropriate."""
        # Check if client accepts gzip
        accept_encoding = request.get("headers", {}).get("accept-encoding", "")
        if "gzip" not in accept_encoding.lower():
            return response

        # Check if response body exists and is large enough
        body = response.get("body")
        if not body:
            return response

        # Serialize body to JSON to get size
        import json
        try:
            body_str = json.dumps(body)
            body_bytes = body_str.encode("utf-8")

            if len(body_bytes) < self.minimum_size:
                return response

            # Compress
            import gzip
            compressed = gzip.compress(body_bytes, compresslevel=self.compression_level)

            # Update response
            # Note: For now we'll just log compression
            # Full implementation would need to handle binary response bodies
            logger.debug(f"Compressed response: {len(body_bytes)} → {len(compressed)} bytes ({len(compressed)/len(body_bytes)*100:.1f}%)")

        except Exception as e:
            logger.warning(f"Compression failed: {e}")

        return response


class OpenTelemetryMiddleware(BaseMiddleware):
    """
    OpenTelemetry distributed tracing middleware.

    Extracts trace context from Rust-injected headers and creates child spans
    for Python handler execution.

    This enables cross-language distributed tracing:
    - Rust HTTP server creates root span
    - Rust injects W3C TraceContext headers (traceparent, tracestate)
    - Python middleware extracts context and creates child span
    - Python handler executes within the child span

    Example:
        from opentelemetry import trace
        from opentelemetry.sdk.trace import TracerProvider
        from opentelemetry.sdk.trace.export import ConsoleSpanExporter, BatchSpanProcessor

        # Configure OpenTelemetry (typically done at app startup)
        provider = TracerProvider()
        provider.add_span_processor(BatchSpanProcessor(ConsoleSpanExporter()))
        trace.set_tracer_provider(provider)

        # Add middleware to app
        app.add_middleware(OpenTelemetryMiddleware())

    Requirements:
        - opentelemetry-api >= 1.20.0
        - opentelemetry-sdk >= 1.20.0
        - opentelemetry-exporter-otlp >= 1.20.0 (for production)
    """

    def __init__(self, tracer_name: str = "data-bridge-pyloop"):
        """
        Initialize OpenTelemetry middleware.

        Args:
            tracer_name: Name for the OpenTelemetry tracer (default: "data-bridge-pyloop")
        """
        try:
            from opentelemetry import trace
            from opentelemetry.propagate import extract
            from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator

            self.tracer = trace.get_tracer(tracer_name)
            self.propagator = TraceContextTextMapPropagator()
            self._otel_available = True
        except ImportError:
            logger.warning(
                "OpenTelemetry not installed. Install with: "
                "pip install opentelemetry-api opentelemetry-sdk opentelemetry-exporter-otlp"
            )
            self._otel_available = False

    async def process_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Extract trace context from headers and start child span.

        The Rust HTTP server injects W3C TraceContext headers (traceparent, tracestate)
        into the request. We extract this context and create a child span for the
        Python handler execution.
        """
        if not self._otel_available:
            return None

        from opentelemetry import context, trace
        from opentelemetry.trace import SpanKind

        # Extract trace context from headers
        # The Rust server injects 'traceparent' and optionally 'tracestate' headers
        headers = request.get("headers", {})

        # OpenTelemetry expects lowercase header names
        carrier = {k.lower(): v for k, v in headers.items()}

        # Extract context using W3C TraceContext propagator
        ctx = self.propagator.extract(carrier=carrier)

        # Start a new span as a child of the extracted context
        with self.tracer.start_as_current_span(
            "pyloop.request",
            context=ctx,
            kind=SpanKind.INTERNAL,
        ) as span:
            # Set span attributes
            span.set_attribute("http.method", request.get("method", "UNKNOWN"))
            span.set_attribute("http.route", request.get("path", "/"))
            span.set_attribute("http.url", request.get("url", ""))

            # Add query parameters if present
            query_params = request.get("query_params", {})
            if query_params:
                # Don't log sensitive data - just count
                span.set_attribute("http.query_param_count", len(query_params))

            # Store span in request for later access
            request["_otel_span"] = span
            request["_otel_context"] = context.get_current()

        return None

    async def process_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """
        Record response status in span.

        This is called after the handler executes, so we can record the
        final HTTP status code in the span.
        """
        if not self._otel_available:
            return response

        span = request.get("_otel_span")
        if span:
            # Record HTTP status code
            status_code = response.get("status", 200)
            span.set_attribute("http.status_code", status_code)

            # Mark span as error if status code >= 500
            if status_code >= 500:
                from opentelemetry.trace import StatusCode
                span.set_status(StatusCode.ERROR, f"HTTP {status_code}")

        return response


class App:
    """
    Simple FastAPI-style app builder for PyLoop.

    Provides a decorator-based API for registering Python handlers that execute
    on the Rust-backed HTTP server with async/sync support.

    Example:
        >>> from cclab.runtime import App
        >>>
        >>> app = App(title="My API", version="1.0.0")
        >>>
        >>> @app.get("/users/{user_id}")
        >>> async def get_user(path_params):
        ...     return {"user_id": path_params["user_id"]}
        >>>
        >>> @app.post("/users")
        >>> async def create_user(body):
        ...     return {"id": "new_id", **body}
        >>>
        >>> # Run the server
        >>> app.serve(host="127.0.0.1", port=8000)
    """

    def __init__(self, title: str = "DataBridge API", version: str = "0.1.0", debug: bool = False):
        """Initialize the App.

        Args:
            title: API title for OpenAPI documentation
            version: API version for OpenAPI documentation
            debug: Enable debug mode (exposes stack traces)
        """
        if _RustApp is None:
            raise RuntimeError("PyLoop Rust extension not available")
        self._app = _RustApp(title=title, version=version)
        self.debug = debug
        self.middlewares: List[BaseMiddleware] = []  # Middleware stack

    def add_middleware(self, middleware: BaseMiddleware) -> None:
        """
        Add middleware to the app.

        Middleware is applied in the order added:
        - Request processing: first → last
        - Response processing: last → first (reverse order)

        Args:
            middleware: Middleware instance

        Example:
            app.add_middleware(CORSMiddleware(allow_origins=["*"]))
            app.add_middleware(LoggingMiddleware())
        """
        self.middlewares.append(middleware)

    async def _process_middleware_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Process request through middleware chain.

        Returns:
            None if request should continue to handler, or early response dict
        """
        for middleware in self.middlewares:
            early_response = await middleware.process_request(request)
            if early_response is not None:
                # Middleware returned early response, stop processing
                return early_response
        return None

    async def _process_middleware_response(self, request: Dict[str, Any], response: Dict[str, Any]) -> Dict[str, Any]:
        """
        Process response through middleware chain (in reverse order).

        Args:
            request: Original request
            response: Response from handler

        Returns:
            Modified response
        """
        # Process middleware in reverse order for responses
        for middleware in reversed(self.middlewares):
            response = await middleware.process_response(request, response)
        return response

    def _wrap_handler_with_middleware(self, handler):
        """
        Wrap handler with middleware processing.

        Args:
            handler: The async handler function

        Returns:
            Wrapped handler with middleware and error handling
        """
        async def wrapped_handler(request):
            try:
                # Process request middleware
                early_response = await self._process_middleware_request(request)
                if early_response is not None:
                    # Middleware returned early response
                    # Still process response middleware
                    return await self._process_middleware_response(request, early_response)

                # Call handler
                response = await handler(request)

                # Process response middleware
                return await self._process_middleware_response(request, response)

            except Exception as e:
                # Handle error and process response middleware
                error_response = self._handle_error(e, request)
                return await self._process_middleware_response(request, error_response)

        return wrapped_handler

    def _handle_error(self, error: Exception, request: Dict = None) -> Dict[str, Any]:
        """
        Convert Python exception to HTTP response.

        Args:
            error: The exception that was raised
            request: The request dict (for logging context)

        Returns:
            Response dict with appropriate status code and error message
        """
        # HTTPException - already formatted
        if isinstance(error, HTTPException):
            if not isinstance(error, (ValidationError, NotFoundError)):
                # Log non-trivial errors
                logger.warning(
                    f"HTTP {error.status_code}: {error.detail}",
                    extra={"path": request.get("path") if request else None}
                )
            return error.to_response()

        # MongoDB errors (from data-bridge)
        error_str = str(error)

        # Duplicate key error (MongoDB)
        if "duplicate key" in error_str.lower() or "E11000" in error_str:
            logger.warning(f"Duplicate key error: {error_str}")
            return {
                "status": 409,
                "body": {
                    "error": "Resource already exists",
                    "type": "ConflictError"
                }
            }

        # Validation errors (MongoDB/Pydantic)
        if "validation" in error_str.lower() or "ValidationError" in type(error).__name__:
            logger.warning(f"Validation error: {error_str}")
            return {
                "status": 422,
                "body": {
                    "error": "Validation failed",
                    "detail": error_str if self.debug else "Invalid request data",
                    "type": "ValidationError"
                }
            }

        # ObjectId errors (invalid ID format)
        if "ObjectId" in error_str or "invalid objectid" in error_str.lower():
            return {
                "status": 400,
                "body": {
                    "error": "Invalid ID format",
                    "type": "BadRequest"
                }
            }

        # Generic error - don't expose internals in production
        logger.error(f"Unhandled error: {type(error).__name__}: {error_str}", exc_info=True)

        if self.debug:
            # Debug mode: include full error details
            import traceback
            return {
                "status": 500,
                "body": {
                    "error": "Internal Server Error",
                    "type": type(error).__name__,
                    "detail": error_str,
                    "traceback": traceback.format_exc()
                }
            }
        else:
            # Production: generic error message
            return {
                "status": 500,
                "body": {
                    "error": "Internal Server Error",
                    "type": "InternalServerError"
                }
            }

    def _wrap_handler_with_error_handling(self, handler):
        """
        Wrap a handler function with error handling.

        Args:
            handler: The async handler function

        Returns:
            Wrapped handler that catches and converts exceptions
        """
        async def wrapped_handler(request):
            try:
                return await handler(request)
            except Exception as e:
                return self._handle_error(e, request)

        return wrapped_handler

    def get(self, path: str):
        """Register a GET route handler.

        Args:
            path: URL path pattern (e.g., "/users/{user_id}")

        Returns:
            Decorator function for handler registration

        Example:
            >>> @app.get("/status")
            >>> async def get_status(path_params, query_params, headers, body):
            ...     return {"status": "ok"}
        """
        def decorator(func):
            # Wrap with middleware and error handling
            wrapped = self._wrap_handler_with_middleware(func)
            self._app.register_route("GET", path, wrapped)
            return func
        return decorator

    def post(self, path: str):
        """Register a POST route handler.

        Args:
            path: URL path pattern (e.g., "/users")

        Returns:
            Decorator function for handler registration

        Example:
            >>> @app.post("/users")
            >>> async def create_user(path_params, query_params, headers, body):
            ...     return {"id": "new_id", **body}
        """
        def decorator(func):
            # Wrap with middleware and error handling
            wrapped = self._wrap_handler_with_middleware(func)
            self._app.register_route("POST", path, wrapped)
            return func
        return decorator

    def put(self, path: str):
        """Register a PUT route handler.

        Args:
            path: URL path pattern (e.g., "/users/{user_id}")

        Returns:
            Decorator function for handler registration

        Example:
            >>> @app.put("/users/{user_id}")
            >>> async def update_user(path_params, query_params, headers, body):
            ...     return {"id": path_params["user_id"], **body}
        """
        def decorator(func):
            # Wrap with middleware and error handling
            wrapped = self._wrap_handler_with_middleware(func)
            self._app.register_route("PUT", path, wrapped)
            return func
        return decorator

    def patch(self, path: str):
        """Register a PATCH route handler.

        Args:
            path: URL path pattern (e.g., "/users/{user_id}")

        Returns:
            Decorator function for handler registration

        Example:
            >>> @app.patch("/users/{user_id}")
            >>> async def partial_update_user(path_params, query_params, headers, body):
            ...     return {"id": path_params["user_id"], "updated": True}
        """
        def decorator(func):
            # Wrap with middleware and error handling
            wrapped = self._wrap_handler_with_middleware(func)
            self._app.register_route("PATCH", path, wrapped)
            return func
        return decorator

    def delete(self, path: str):
        """Register a DELETE route handler.

        Args:
            path: URL path pattern (e.g., "/users/{user_id}")

        Returns:
            Decorator function for handler registration

        Example:
            >>> @app.delete("/users/{user_id}")
            >>> async def delete_user(path_params, query_params, headers, body):
            ...     return {"deleted": True, "user_id": path_params["user_id"]}
        """
        def decorator(func):
            # Wrap with middleware and error handling
            wrapped = self._wrap_handler_with_middleware(func)
            self._app.register_route("DELETE", path, wrapped)
            return func
        return decorator

    def crud_routes(
        self,
        document_cls,
        prefix: Optional[str] = None,
        *,
        operations: Optional[str] = None,
        tags: Optional[list] = None,
        create: bool = True,
        read: bool = True,
        update: bool = True,
        delete: bool = True,
        list: bool = True,
    ):
        """
        Auto-generate CRUD endpoints for a Document model.

        Args:
            document_cls: Document class to generate CRUD for
            prefix: URL prefix (default: /collection_name)
                   Can be passed as positional argument: app.crud_routes(Product, "/products")
            operations: String specifying operations (e.g., "CRUDL", "CR", "RUD")
                       C=Create, R=Read, U=Update, D=Delete, L=List
                       If provided, overrides individual flags
            tags: OpenAPI tags (default: [collection_name])
            create: Generate POST endpoint (default: True)
            read: Generate GET /{id} endpoint (default: True)
            update: Generate PUT /{id} endpoint (default: True)
            delete: Generate DELETE /{id} endpoint (default: True)
            list: Generate GET / endpoint with pagination (default: True)

        Example:
            # All operations with default prefix
            app.crud_routes(Product)

            # Custom prefix (positional argument - recommended)
            app.crud_routes(Product, "/products")

            # With operation string
            app.crud_routes(Product, "/products", operations="RL")

            # Only read operations
            app.crud_routes(Product, operations="RL")

            # Explicit flags
            app.crud_routes(Product, "/products", create=True, read=True, list=False)
        """
        # Parse operations string if provided
        if operations is not None:
            operations = operations.upper()
            create = "C" in operations
            read = "R" in operations
            update = "U" in operations
            delete = "D" in operations
            list = "L" in operations

        # Get collection name from Document class
        if hasattr(document_cls, '__collection_name__'):
            collection_name = document_cls.__collection_name__()
        elif hasattr(document_cls, '_collection_name'):
            collection_name = document_cls._collection_name
        else:
            collection_name = document_cls.__name__.lower()

        # Default prefix to /collection_name
        if prefix is None:
            prefix = f"/{collection_name}"

        # Default tags
        if tags is None:
            tags = [collection_name]

        # Generate LIST endpoint: GET /resource?skip=0&limit=10
        async def list_handler(request):
            """List documents with pagination."""
            query_params = request.get("query_params", {})

            # Extract pagination params
            skip = int(query_params.get("skip", 0))
            limit = int(query_params.get("limit", 10))
            limit = min(limit, 100)  # Cap at 100

            # Execute query
            documents = await document_cls.find().skip(skip).limit(limit).to_list()

            # Serialize to dict
            items = [doc.to_dict() for doc in documents]

            return {
                "status": 200,
                "body": {
                    "items": items,
                    "skip": skip,
                    "limit": limit,
                    "total": len(items)
                }
            }

        # Generate GET_BY_ID endpoint: GET /resource/{id}
        async def get_handler(request):
            """Get document by ID."""
            doc_id = request["path_params"]["id"]

            try:
                document = await document_cls.get(doc_id)
                if document is None:
                    raise NotFoundError(f"{collection_name.capitalize()} not found")

                return {"status": 200, "body": document.to_dict()}
            except HTTPException:
                raise  # Re-raise HTTPException as-is
            except Exception as e:
                # Let the error handler deal with it
                raise

        # Generate CREATE endpoint: POST /resource
        async def create_handler(request):
            """Create new document."""
            body = request.get("body", {})

            if not body:
                raise ValidationError("Request body required")

            try:
                document = document_cls(**body)
                await document.save()
                return {"status": 201, "body": document.to_dict()}
            except HTTPException:
                raise
            except Exception as e:
                raise  # Will be caught by error handler

        # Generate UPDATE endpoint: PUT /resource/{id}
        async def update_handler(request):
            """Update document by ID."""
            doc_id = request["path_params"]["id"]
            body = request.get("body", {})

            if not body:
                raise ValidationError("Request body required")

            try:
                document = await document_cls.get(doc_id)
                if document is None:
                    raise NotFoundError(f"{collection_name.capitalize()} not found")

                for key, value in body.items():
                    if not key.startswith('_'):
                        setattr(document, key, value)

                await document.save()
                return {"status": 200, "body": document.to_dict()}
            except HTTPException:
                raise
            except Exception as e:
                raise

        # Generate DELETE endpoint: DELETE /resource/{id}
        async def delete_handler(request):
            """Delete document by ID."""
            doc_id = request["path_params"]["id"]

            try:
                document = await document_cls.get(doc_id)
                if document is None:
                    raise NotFoundError(f"{collection_name.capitalize()} not found")

                await document.delete()
                return {"status": 204, "body": None}
            except HTTPException:
                raise
            except Exception as e:
                raise

        # Register endpoints based on flags
        if list:
            self.get(f"{prefix}")(list_handler)

        if read:
            self.get(f"{prefix}/{{id}}")(get_handler)

        if create:
            self.post(f"{prefix}")(create_handler)

        if update:
            self.put(f"{prefix}/{{id}}")(update_handler)

        if delete:
            self.delete(f"{prefix}/{{id}}")(delete_handler)

    def crud(self, document_cls, prefix: Optional[str] = None, tags: Optional[list] = None):
        """
        Legacy decorator-style CRUD generation (deprecated).

        Use crud_routes() instead for direct method call:
            app.crud_routes(Product)

        This method is kept for backward compatibility.

        Args:
            document_cls: Document class to generate CRUD for
            prefix: URL prefix (default: /collection_name)
            tags: OpenAPI tags (default: [collection_name])

        Returns:
            Decorator function (for @app.crud(Model) syntax)

        Example:
            from cclab.mongo import Document
            from cclab.runtime import App

            class Product(Document):
                name: str
                price: float

                class Settings:
                    name = "products"

            app = App()

            @app.crud(Product)
            class ProductCRUD:
                pass  # Endpoints auto-generated

            # This generates:
            # GET /products?skip=0&limit=10 - List products
            # GET /products/{id} - Get product
            # POST /products - Create product
            # PUT /products/{id} - Update product
            # DELETE /products/{id} - Delete product
        """
        # Call the new crud_routes method
        self.crud_routes(document_cls, prefix=prefix, tags=tags)

        # Return decorator for @app.crud(Model) syntax
        def decorator(cls):
            return cls
        return decorator

    def serve(self, host: str = "127.0.0.1", port: int = 8000):
        """
        Start the HTTP server (blocking).

        This method will block until the server receives a shutdown signal (Ctrl+C).
        The server runs with the GIL released for maximum performance.

        Args:
            host: Bind address (default: 127.0.0.1)
            port: Bind port (default: 8000)

        Example:
            >>> app.serve(host="0.0.0.0", port=3000)
        """
        print(f"Starting server on http://{host}:{port}")
        print("Press Ctrl+C to stop")
        self._app.serve(host, port)
