"""Tests for PyLoop middleware."""

import pytest
from cclab.orbit import (
    App,
    BaseMiddleware,
    CORSMiddleware,
    LoggingMiddleware,
    CompressionMiddleware
)

def test_base_middleware_abstract():
    """Test that BaseMiddleware is abstract."""
    with pytest.raises(TypeError):
        BaseMiddleware()

def test_cors_middleware_creation():
    """Test CORSMiddleware creation."""
    cors = CORSMiddleware(
        allow_origins=["https://example.com"],
        allow_methods=["GET", "POST"],
        allow_headers=["Content-Type"],
        allow_credentials=True,
        max_age=600
    )

    assert cors.allow_origins == ["https://example.com"]
    assert cors.allow_methods == ["GET", "POST"]
    assert cors.allow_credentials is True

def test_cors_middleware_wildcard():
    """Test CORS with wildcard origin."""
    cors = CORSMiddleware(allow_origins=["*"])
    assert cors.allow_all_origins is True

def test_cors_middleware_defaults():
    """Test CORS with default values."""
    cors = CORSMiddleware()
    assert cors.allow_origins == ["*"]
    assert cors.allow_all_origins is True
    assert "GET" in cors.allow_methods
    assert "POST" in cors.allow_methods
    assert cors.max_age == 600

def test_cors_origin_allowed():
    """Test origin checking."""
    cors = CORSMiddleware(allow_origins=["https://example.com", "https://app.example.com"])

    assert cors._is_origin_allowed("https://example.com") is True
    assert cors._is_origin_allowed("https://app.example.com") is True
    assert cors._is_origin_allowed("https://evil.com") is False

def test_cors_origin_allowed_wildcard():
    """Test wildcard origin checking."""
    cors = CORSMiddleware(allow_origins=["*"])

    assert cors._is_origin_allowed("https://example.com") is True
    assert cors._is_origin_allowed("https://evil.com") is True

def test_logging_middleware_creation():
    """Test LoggingMiddleware creation."""
    log_mid = LoggingMiddleware(
        log_request_body=True,
        log_response_body=True
    )

    assert log_mid.log_request_body is True
    assert log_mid.log_response_body is True

def test_logging_middleware_defaults():
    """Test LoggingMiddleware with default values."""
    log_mid = LoggingMiddleware()

    assert log_mid.log_request_body is False
    assert log_mid.log_response_body is False
    assert log_mid.logger_instance is not None

def test_compression_middleware_creation():
    """Test CompressionMiddleware creation."""
    comp = CompressionMiddleware(
        minimum_size=1000,
        compression_level=9
    )

    assert comp.minimum_size == 1000
    assert comp.compression_level == 9

def test_compression_middleware_defaults():
    """Test CompressionMiddleware with default values."""
    comp = CompressionMiddleware()

    assert comp.minimum_size == 500
    assert comp.compression_level == 6

def test_app_add_middleware():
    """Test adding middleware to app."""
    app = App()

    cors = CORSMiddleware(allow_origins=["*"])
    app.add_middleware(cors)

    assert len(app.middlewares) == 1
    assert app.middlewares[0] is cors

def test_app_multiple_middlewares():
    """Test adding multiple middlewares."""
    app = App()

    cors = CORSMiddleware()
    logging_mid = LoggingMiddleware()

    app.add_middleware(cors)
    app.add_middleware(logging_mid)

    assert len(app.middlewares) == 2
    assert app.middlewares[0] is cors
    assert app.middlewares[1] is logging_mid

def test_app_middleware_order():
    """Test middleware order is preserved."""
    app = App()

    cors = CORSMiddleware()
    logging_mid = LoggingMiddleware()
    comp = CompressionMiddleware()

    app.add_middleware(cors)
    app.add_middleware(logging_mid)
    app.add_middleware(comp)

    assert len(app.middlewares) == 3
    assert app.middlewares[0] is cors
    assert app.middlewares[1] is logging_mid
    assert app.middlewares[2] is comp

@pytest.mark.asyncio
async def test_cors_preflight_request():
    """Test CORS preflight request handling."""
    cors = CORSMiddleware(
        allow_origins=["https://example.com"],
        allow_methods=["GET", "POST"],
        allow_headers=["Content-Type"]
    )

    # Preflight request
    request = {
        "method": "OPTIONS",
        "path": "/api/data",
        "headers": {
            "origin": "https://example.com",
            "access-control-request-method": "POST",
            "access-control-request-headers": "Content-Type"
        }
    }

    response = await cors.process_request(request)

    assert response is not None
    assert response["status"] == 204
    assert "Access-Control-Allow-Origin" in response["headers"]
    assert response["headers"]["Access-Control-Allow-Origin"] == "https://example.com"

@pytest.mark.asyncio
async def test_cors_preflight_rejected():
    """Test CORS preflight with disallowed origin."""
    cors = CORSMiddleware(allow_origins=["https://example.com"])

    # Preflight from disallowed origin
    request = {
        "method": "OPTIONS",
        "path": "/api/data",
        "headers": {
            "origin": "https://evil.com"
        }
    }

    response = await cors.process_request(request)
    # Should not return early response for disallowed origin
    assert response is None

@pytest.mark.asyncio
async def test_cors_add_response_headers():
    """Test CORS adds headers to response."""
    cors = CORSMiddleware(allow_origins=["https://example.com"])

    request = {
        "method": "GET",
        "path": "/api/data",
        "headers": {
            "origin": "https://example.com"
        }
    }

    response = {
        "status": 200,
        "body": {"data": "test"}
    }

    modified_response = await cors.process_response(request, response)

    assert "headers" in modified_response
    assert "Access-Control-Allow-Origin" in modified_response["headers"]
    assert modified_response["headers"]["Access-Control-Allow-Origin"] == "https://example.com"

@pytest.mark.asyncio
async def test_logging_middleware_timing():
    """Test logging middleware tracks request timing."""
    import logging
    log_mid = LoggingMiddleware(logger_instance=logging.getLogger("test"))

    request = {
        "method": "GET",
        "path": "/test",
        "query_params": {}
    }

    # Process request (adds start time)
    await log_mid.process_request(request)

    assert "_middleware_start_time" in request
    assert isinstance(request["_middleware_start_time"], float)

    # Process response (calculates duration)
    response = {"status": 200, "body": {}}
    await log_mid.process_response(request, response)

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
