"""Integration tests for PyLoop middleware with actual app."""

import pytest
from cclab.orbit import (
    App,
    BaseMiddleware,
    CORSMiddleware,
    LoggingMiddleware,
)


class CustomTestMiddleware(BaseMiddleware):
    """Custom middleware for testing."""

    def __init__(self):
        self.request_processed = False
        self.response_processed = False

    async def process_request(self, request):
        self.request_processed = True
        # Add custom header to request
        request["custom_data"] = "middleware_data"
        return None  # Continue to handler

    async def process_response(self, request, response):
        self.response_processed = True
        # Add custom header to response
        if "headers" not in response:
            response["headers"] = {}
        response["headers"]["X-Custom-Middleware"] = "processed"
        return response


class EarlyResponseMiddleware(BaseMiddleware):
    """Middleware that returns early response."""

    async def process_request(self, request):
        if request.get("path") == "/blocked":
            return {
                "status": 403,
                "body": {"error": "Blocked by middleware"}
            }
        return None

    async def process_response(self, request, response):
        return response


@pytest.mark.asyncio
async def test_middleware_processes_request_and_response():
    """Test that middleware processes both request and response."""
    app = App()
    custom = CustomTestMiddleware()
    app.add_middleware(custom)

    @app.get("/test")
    async def handler(request):
        # Check that middleware added custom data
        assert request.get("custom_data") == "middleware_data"
        return {"status": 200, "body": {"message": "ok"}}

    # Simulate request
    request = {"method": "GET", "path": "/test", "headers": {}}

    # Get the wrapped handler
    wrapped = app._wrap_handler_with_middleware(handler)
    response = await wrapped(request)

    # Verify middleware was called
    assert custom.request_processed is True
    assert custom.response_processed is True

    # Verify custom header was added
    assert "headers" in response
    assert response["headers"]["X-Custom-Middleware"] == "processed"


@pytest.mark.asyncio
async def test_middleware_early_response():
    """Test that middleware can return early response."""
    app = App()
    early = EarlyResponseMiddleware()
    app.add_middleware(early)

    handler_called = False

    @app.get("/blocked")
    async def handler(request):
        nonlocal handler_called
        handler_called = True
        return {"status": 200, "body": {"message": "ok"}}

    # Simulate blocked request
    request = {"method": "GET", "path": "/blocked", "headers": {}}

    # Get the wrapped handler
    wrapped = app._wrap_handler_with_middleware(handler)
    response = await wrapped(request)

    # Verify handler was NOT called
    assert handler_called is False

    # Verify early response was returned
    assert response["status"] == 403
    assert response["body"]["error"] == "Blocked by middleware"


@pytest.mark.asyncio
async def test_middleware_order():
    """Test that middleware is applied in correct order."""
    app = App()
    order = []

    class FirstMiddleware(BaseMiddleware):
        async def process_request(self, request):
            order.append("first_request")
            return None

        async def process_response(self, request, response):
            order.append("first_response")
            return response

    class SecondMiddleware(BaseMiddleware):
        async def process_request(self, request):
            order.append("second_request")
            return None

        async def process_response(self, request, response):
            order.append("second_response")
            return response

    app.add_middleware(FirstMiddleware())
    app.add_middleware(SecondMiddleware())

    @app.get("/test")
    async def handler(request):
        order.append("handler")
        return {"status": 200, "body": {}}

    # Simulate request
    request = {"method": "GET", "path": "/test", "headers": {}}
    wrapped = app._wrap_handler_with_middleware(handler)
    await wrapped(request)

    # Verify order: request first->second, handler, response second->first (reverse)
    assert order == [
        "first_request",
        "second_request",
        "handler",
        "second_response",
        "first_response"
    ]


@pytest.mark.asyncio
async def test_cors_middleware_integration():
    """Test CORS middleware with app."""
    app = App()
    app.add_middleware(CORSMiddleware(
        allow_origins=["https://example.com"],
        allow_methods=["GET", "POST"]
    ))

    @app.get("/api/data")
    async def handler(request):
        return {"status": 200, "body": {"data": "test"}}

    # Test regular request with origin
    request = {
        "method": "GET",
        "path": "/api/data",
        "headers": {"origin": "https://example.com"}
    }

    wrapped = app._wrap_handler_with_middleware(handler)
    response = await wrapped(request)

    # Verify CORS headers
    assert "headers" in response
    assert "Access-Control-Allow-Origin" in response["headers"]
    assert response["headers"]["Access-Control-Allow-Origin"] == "https://example.com"


@pytest.mark.asyncio
async def test_cors_preflight_integration():
    """Test CORS preflight with app."""
    app = App()
    app.add_middleware(CORSMiddleware(
        allow_origins=["https://example.com"],
        allow_methods=["GET", "POST"]
    ))

    handler_called = False

    @app.post("/api/data")
    async def handler(request):
        nonlocal handler_called
        handler_called = True
        return {"status": 200, "body": {"data": "created"}}

    # Test preflight request
    request = {
        "method": "OPTIONS",
        "path": "/api/data",
        "headers": {
            "origin": "https://example.com",
            "access-control-request-method": "POST"
        }
    }

    wrapped = app._wrap_handler_with_middleware(handler)
    response = await wrapped(request)

    # Verify handler was NOT called (preflight returns early)
    assert handler_called is False

    # Verify preflight response
    assert response["status"] == 204
    assert "Access-Control-Allow-Origin" in response["headers"]


@pytest.mark.asyncio
async def test_middleware_with_error_handling():
    """Test that middleware processes responses from error handler."""
    app = App()
    custom = CustomTestMiddleware()
    app.add_middleware(custom)

    @app.get("/error")
    async def handler(request):
        raise ValueError("Test error")

    # Simulate request
    request = {"method": "GET", "path": "/error", "headers": {}}
    wrapped = app._wrap_handler_with_middleware(handler)
    response = await wrapped(request)

    # Verify error was handled
    assert response["status"] == 500

    # Verify middleware still processed response
    assert custom.response_processed is True
    assert "X-Custom-Middleware" in response["headers"]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
