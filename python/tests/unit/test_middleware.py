"""
Tests for middleware system.
"""
import pytest
import asyncio
import logging
from cclab.quasar import (
    BaseMiddleware,
    MiddlewareStack,
    TimingMiddleware,
    LoggingMiddleware,
    CORSMiddleware,
    CORSConfig,
    App,
    Response,
)


class TestBaseMiddleware:
    """Test BaseMiddleware abstract class"""

    def test_base_middleware_cannot_be_instantiated(self):
        """Test that BaseMiddleware cannot be instantiated directly"""
        with pytest.raises(TypeError):
            BaseMiddleware()

    def test_base_middleware_requires_call_method(self):
        """Test that BaseMiddleware subclasses must implement __call__"""
        class IncompleteMiddleware(BaseMiddleware):
            pass

        with pytest.raises(TypeError):
            IncompleteMiddleware()


class TestMiddlewareStack:
    """Test MiddlewareStack class"""

    def test_middleware_stack_init(self):
        """Test MiddlewareStack initialization"""
        stack = MiddlewareStack()
        assert stack._middlewares == []

    def test_middleware_stack_add(self):
        """Test adding middleware to stack"""
        stack = MiddlewareStack()

        class TestMiddleware(BaseMiddleware):
            async def __call__(self, request, call_next):
                return await call_next(request)

        middleware = TestMiddleware()
        stack.add(middleware)
        assert len(stack._middlewares) == 1
        assert stack._middlewares[0] is middleware

    def test_middleware_stack_lifo_order(self):
        """Test that middleware is added in LIFO order (last added runs first)"""
        stack = MiddlewareStack()

        class Middleware1(BaseMiddleware):
            async def __call__(self, request, call_next):
                request["order"].append(1)
                return await call_next(request)

        class Middleware2(BaseMiddleware):
            async def __call__(self, request, call_next):
                request["order"].append(2)
                return await call_next(request)

        # Add in order 1, 2
        stack.add(Middleware1())
        stack.add(Middleware2())

        # Middleware 2 should be at position 0 (runs first)
        assert isinstance(stack._middlewares[0], Middleware2)
        assert isinstance(stack._middlewares[1], Middleware1)

    @pytest.mark.asyncio
    async def test_middleware_stack_wrap(self):
        """Test wrapping a handler with middleware"""
        stack = MiddlewareStack()

        class TestMiddleware(BaseMiddleware):
            async def __call__(self, request, call_next):
                request["modified"] = True
                return await call_next(request)

        stack.add(TestMiddleware())

        async def handler(request):
            return {"result": request.get("modified", False)}

        wrapped = stack.wrap(handler)
        result = await wrapped({"test": True})
        assert result["result"] is True

    @pytest.mark.asyncio
    async def test_middleware_execution_order(self):
        """Test that middleware executes in LIFO order"""
        stack = MiddlewareStack()
        execution_order = []

        class Middleware1(BaseMiddleware):
            async def __call__(self, request, call_next):
                execution_order.append("m1_before")
                response = await call_next(request)
                execution_order.append("m1_after")
                return response

        class Middleware2(BaseMiddleware):
            async def __call__(self, request, call_next):
                execution_order.append("m2_before")
                response = await call_next(request)
                execution_order.append("m2_after")
                return response

        # Add middleware 1 first, then 2
        stack.add(Middleware1())
        stack.add(Middleware2())

        async def handler(request):
            execution_order.append("handler")
            return "done"

        wrapped = stack.wrap(handler)
        await wrapped({})

        # Middleware 2 runs first (outer), then middleware 1 (inner), then handler
        assert execution_order == [
            "m2_before",
            "m1_before",
            "handler",
            "m1_after",
            "m2_after",
        ]

    @pytest.mark.asyncio
    async def test_empty_middleware_stack(self):
        """Test that empty middleware stack works"""
        stack = MiddlewareStack()

        async def handler(request):
            return {"result": "ok"}

        wrapped = stack.wrap(handler)
        result = await wrapped({"test": True})
        assert result["result"] == "ok"


class TestTimingMiddleware:
    """Test TimingMiddleware class"""

    @pytest.mark.asyncio
    async def test_timing_middleware_adds_header(self):
        """Test that TimingMiddleware adds timing header"""
        middleware = TimingMiddleware()

        class MockResponse:
            def __init__(self):
                self.headers = {}

        async def handler(request):
            return MockResponse()

        response = await middleware({"test": True}, handler)
        assert "X-Response-Time" in response.headers
        assert response.headers["X-Response-Time"].endswith("ms")

    @pytest.mark.asyncio
    async def test_timing_middleware_without_headers(self):
        """Test TimingMiddleware with response that doesn't have headers"""
        middleware = TimingMiddleware()

        async def handler(request):
            return {"result": "ok"}

        # Should not raise error if response doesn't have headers attribute
        response = await middleware({"test": True}, handler)
        assert response == {"result": "ok"}

    @pytest.mark.asyncio
    async def test_timing_middleware_measures_time(self):
        """Test that TimingMiddleware measures execution time"""
        middleware = TimingMiddleware()

        class MockResponse:
            def __init__(self):
                self.headers = {}

        async def handler(request):
            await asyncio.sleep(0.01)  # 10ms
            return MockResponse()

        response = await middleware({"test": True}, handler)
        # Extract time from header (format: "XX.XXms")
        time_str = response.headers["X-Response-Time"]
        time_ms = float(time_str.replace("ms", ""))
        # Should be at least 10ms
        assert time_ms >= 10.0


class TestLoggingMiddleware:
    """Test LoggingMiddleware class"""

    @pytest.mark.asyncio
    async def test_logging_middleware_logs_request_and_response(self):
        """Test that LoggingMiddleware logs request and response"""
        # Create a logger with custom handler to capture logs
        logger = logging.getLogger("test_logger")
        logger.setLevel(logging.INFO)

        # Store log records
        log_records = []

        class ListHandler(logging.Handler):
            def emit(self, record):
                log_records.append(record.getMessage())

        handler = ListHandler()
        logger.addHandler(handler)

        middleware = LoggingMiddleware(logger=logger)

        class MockResponse:
            status_code = 200

        async def handler_func(request):
            return MockResponse()

        await middleware({"method": "GET", "path": "/test"}, handler_func)

        assert len(log_records) == 2
        assert "Request: GET /test" in log_records[0]
        assert "Response: 200" in log_records[1]

        # Cleanup
        logger.removeHandler(handler)

    @pytest.mark.asyncio
    async def test_logging_middleware_without_logger(self):
        """Test LoggingMiddleware without logger (no-op)"""
        middleware = LoggingMiddleware(logger=None)

        async def handler(request):
            return {"result": "ok"}

        # Should work without error even without logger
        response = await middleware({"method": "GET", "path": "/test"}, handler)
        assert response["result"] == "ok"

    @pytest.mark.asyncio
    async def test_logging_middleware_default_status_code(self):
        """Test LoggingMiddleware uses default status code if not present"""
        logger = logging.getLogger("test_logger_2")
        logger.setLevel(logging.INFO)

        log_records = []

        class ListHandler(logging.Handler):
            def emit(self, record):
                log_records.append(record.getMessage())

        handler = ListHandler()
        logger.addHandler(handler)

        middleware = LoggingMiddleware(logger=logger)

        async def handler_func(request):
            return {"result": "ok"}  # No status_code attribute

        await middleware({"method": "POST", "path": "/data"}, handler_func)

        assert len(log_records) == 2
        assert "Request: POST /data" in log_records[0]
        assert "Response: 200" in log_records[1]  # Default status code

        # Cleanup
        logger.removeHandler(handler)


class TestMiddlewareChaining:
    """Test multiple middleware working together"""

    @pytest.mark.asyncio
    async def test_multiple_middleware_chain(self):
        """Test chaining multiple middleware together"""
        stack = MiddlewareStack()

        class AddHeaderMiddleware(BaseMiddleware):
            def __init__(self, header_name):
                self.header_name = header_name

            async def __call__(self, request, call_next):
                response = await call_next(request)
                if hasattr(response, "headers"):
                    response.headers[self.header_name] = "added"
                return response

        class ModifyRequestMiddleware(BaseMiddleware):
            async def __call__(self, request, call_next):
                request["modified"] = True
                return await call_next(request)

        stack.add(ModifyRequestMiddleware())
        stack.add(AddHeaderMiddleware("X-Custom-1"))
        stack.add(AddHeaderMiddleware("X-Custom-2"))

        class MockResponse:
            def __init__(self):
                self.headers = {}

        async def handler(request):
            response = MockResponse()
            response.request_modified = request.get("modified", False)
            return response

        wrapped = stack.wrap(handler)
        result = await wrapped({"test": True})

        assert result.request_modified is True
        assert "X-Custom-1" in result.headers
        assert "X-Custom-2" in result.headers

    @pytest.mark.asyncio
    async def test_error_propagation(self):
        """Test that errors propagate through middleware chain"""
        stack = MiddlewareStack()

        class ErrorCatchingMiddleware(BaseMiddleware):
            async def __call__(self, request, call_next):
                try:
                    return await call_next(request)
                except ValueError as e:
                    return {"error": str(e)}

        stack.add(ErrorCatchingMiddleware())

        async def handler(request):
            raise ValueError("Test error")

        wrapped = stack.wrap(handler)
        result = await wrapped({})
        assert result["error"] == "Test error"


class TestAppIntegration:
    """Test middleware integration with App class"""

    def test_app_has_add_middleware_method(self):
        """Test that App has add_middleware method"""
        app = App()
        assert hasattr(app, "add_middleware")
        assert callable(app.add_middleware)

    def test_app_add_middleware(self):
        """Test adding middleware to App"""
        app = App()

        class TestMiddleware(BaseMiddleware):
            async def __call__(self, request, call_next):
                return await call_next(request)

        middleware = TestMiddleware()
        app.add_middleware(middleware)

        assert len(app._middleware_stack._middlewares) == 1
        assert app._middleware_stack._middlewares[0] is middleware

    def test_app_add_timing_middleware(self):
        """Test adding TimingMiddleware to App"""
        app = App()
        app.add_middleware(TimingMiddleware())
        assert len(app._middleware_stack._middlewares) == 1
        assert isinstance(app._middleware_stack._middlewares[0], TimingMiddleware)

    def test_app_add_logging_middleware(self):
        """Test adding LoggingMiddleware to App"""
        app = App()
        logger = logging.getLogger("test_app")
        app.add_middleware(LoggingMiddleware(logger=logger))
        assert len(app._middleware_stack._middlewares) == 1
        assert isinstance(app._middleware_stack._middlewares[0], LoggingMiddleware)


class TestCORSConfig:
    """Test CORSConfig dataclass"""

    def test_cors_config_defaults(self):
        """Test CORSConfig default values"""
        config = CORSConfig()
        assert config.allow_origins == {"*"}
        assert config.allow_methods == {"GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"}
        assert config.allow_headers == {"*"}
        assert config.allow_credentials is False
        assert config.expose_headers == set()
        assert config.max_age == 600

    def test_cors_config_custom(self):
        """Test CORSConfig with custom values"""
        config = CORSConfig(
            allow_origins={"https://example.com"},
            allow_methods={"GET", "POST"},
            allow_headers={"Content-Type"},
            allow_credentials=True,
            expose_headers={"X-Custom"},
            max_age=3600,
        )
        assert config.allow_origins == {"https://example.com"}
        assert config.allow_methods == {"GET", "POST"}
        assert config.allow_headers == {"Content-Type"}
        assert config.allow_credentials is True
        assert config.expose_headers == {"X-Custom"}
        assert config.max_age == 3600


class TestCORSMiddleware:
    """Test CORSMiddleware class"""

    def test_cors_middleware_init_defaults(self):
        """Test CORSMiddleware initialization with defaults"""
        middleware = CORSMiddleware()
        assert middleware.config.allow_origins == {"*"}
        assert middleware.config.allow_methods == {"GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"}
        assert middleware.config.allow_headers == {"*"}
        assert middleware.config.allow_credentials is False
        assert middleware.config.expose_headers == set()
        assert middleware.config.max_age == 600

    def test_cors_middleware_init_custom(self):
        """Test CORSMiddleware initialization with custom values"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            allow_methods={"GET", "POST"},
            allow_headers={"Content-Type"},
            allow_credentials=True,
            expose_headers={"X-Custom"},
            max_age=3600,
        )
        assert middleware.config.allow_origins == {"https://example.com"}
        assert middleware.config.allow_methods == {"GET", "POST"}
        assert middleware.config.allow_headers == {"Content-Type"}
        assert middleware.config.allow_credentials is True
        assert middleware.config.expose_headers == {"X-Custom"}
        assert middleware.config.max_age == 3600

    def test_is_origin_allowed_wildcard(self):
        """Test origin check with wildcard"""
        middleware = CORSMiddleware(allow_origins={"*"})
        assert middleware._is_origin_allowed("https://example.com") is True
        assert middleware._is_origin_allowed("https://any-domain.com") is True

    def test_is_origin_allowed_specific(self):
        """Test origin check with specific origins"""
        middleware = CORSMiddleware(allow_origins={"https://example.com", "https://test.com"})
        assert middleware._is_origin_allowed("https://example.com") is True
        assert middleware._is_origin_allowed("https://test.com") is True
        assert middleware._is_origin_allowed("https://other.com") is False

    @pytest.mark.asyncio
    async def test_cors_headers_wildcard_origin(self):
        """Test CORS headers with wildcard origin (allow all)"""
        middleware = CORSMiddleware()

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert response.headers["Access-Control-Allow-Origin"] == "*"

    @pytest.mark.asyncio
    async def test_cors_headers_specific_origin(self):
        """Test CORS headers with specific allowed origin"""
        middleware = CORSMiddleware(allow_origins={"https://example.com"})

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert response.headers["Access-Control-Allow-Origin"] == "https://example.com"

    @pytest.mark.asyncio
    async def test_cors_origin_not_allowed(self):
        """Test that disallowed origin doesn't get CORS headers"""
        middleware = CORSMiddleware(allow_origins={"https://example.com"})

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://evil.com"}}
        response = await middleware(request, handler)

        assert "Access-Control-Allow-Origin" not in response.headers

    @pytest.mark.asyncio
    async def test_cors_credentials_header(self):
        """Test that credentials header is added when enabled"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            allow_credentials=True
        )

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert response.headers["Access-Control-Allow-Credentials"] == "true"
        assert response.headers["Access-Control-Allow-Origin"] == "https://example.com"

    @pytest.mark.asyncio
    async def test_cors_credentials_no_wildcard(self):
        """Test that credentials mode requires specific origin (not wildcard)"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            allow_credentials=True
        )

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        # With credentials, should use specific origin not wildcard
        assert response.headers["Access-Control-Allow-Origin"] == "https://example.com"
        assert response.headers["Access-Control-Allow-Credentials"] == "true"

    @pytest.mark.asyncio
    async def test_cors_expose_headers(self):
        """Test that expose headers are added"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            expose_headers={"X-Custom-1", "X-Custom-2"}
        )

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert "Access-Control-Expose-Headers" in response.headers
        expose_headers = set(response.headers["Access-Control-Expose-Headers"].split(", "))
        assert expose_headers == {"X-Custom-1", "X-Custom-2"}

    @pytest.mark.asyncio
    async def test_cors_preflight_options(self):
        """Test preflight OPTIONS request handling"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            allow_methods={"GET", "POST"},
            allow_headers={"Content-Type"},
            max_age=3600
        )

        async def handler(request):
            # This should not be called for OPTIONS preflight
            return Response(content="should not reach here", status_code=200)

        request = {"method": "OPTIONS", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        # Check preflight response
        assert response.status_code == 204
        assert response.content == ""
        assert response.headers["Access-Control-Allow-Origin"] == "https://example.com"
        assert "Access-Control-Allow-Methods" in response.headers
        assert "Access-Control-Allow-Headers" in response.headers
        assert response.headers["Access-Control-Max-Age"] == "3600"

    @pytest.mark.asyncio
    async def test_cors_preflight_wildcard_headers(self):
        """Test preflight with wildcard headers"""
        middleware = CORSMiddleware(allow_origins={"https://example.com"})

        async def handler(request):
            return Response(content="test", status_code=200)

        request = {"method": "OPTIONS", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert response.headers["Access-Control-Allow-Headers"] == "*"

    @pytest.mark.asyncio
    async def test_cors_preflight_specific_headers(self):
        """Test preflight with specific headers"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            allow_headers={"Content-Type", "Authorization"}
        )

        async def handler(request):
            return Response(content="test", status_code=200)

        request = {"method": "OPTIONS", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        allow_headers = set(response.headers["Access-Control-Allow-Headers"].split(", "))
        assert allow_headers == {"Content-Type", "Authorization"}

    @pytest.mark.asyncio
    async def test_cors_no_origin_header(self):
        """Test request without origin header"""
        middleware = CORSMiddleware()

        async def handler(request):
            response = Response(content="test", status_code=200)
            return response

        request = {"method": "GET", "headers": {}}
        response = await middleware(request, handler)

        # No origin, so no CORS headers should be added
        assert "Access-Control-Allow-Origin" not in response.headers

    @pytest.mark.asyncio
    async def test_cors_response_without_headers_attribute(self):
        """Test CORS middleware with response that doesn't have headers"""
        middleware = CORSMiddleware()

        async def handler(request):
            return {"result": "ok"}  # Dict response without headers attribute

        request = {"method": "GET", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        # Should not raise error
        assert response == {"result": "ok"}

    @pytest.mark.asyncio
    async def test_cors_custom_max_age(self):
        """Test custom max age for preflight cache"""
        middleware = CORSMiddleware(
            allow_origins={"https://example.com"},
            max_age=7200
        )

        async def handler(request):
            return Response(content="test", status_code=200)

        request = {"method": "OPTIONS", "headers": {"origin": "https://example.com"}}
        response = await middleware(request, handler)

        assert response.headers["Access-Control-Max-Age"] == "7200"

    def test_cors_middleware_with_app(self):
        """Test adding CORSMiddleware to App"""
        app = App()
        middleware = CORSMiddleware(allow_origins={"https://example.com"})
        app.add_middleware(middleware)

        assert len(app._middleware_stack._middlewares) == 1
        assert isinstance(app._middleware_stack._middlewares[0], CORSMiddleware)
