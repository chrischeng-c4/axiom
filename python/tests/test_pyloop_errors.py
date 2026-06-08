"""Tests for PyLoop error handling."""

import pytest
from cclab.orbit import App, HTTPException, ValidationError, NotFoundError

def test_http_exception_basic():
    """Test HTTPException creation."""
    exc = HTTPException(404, "Not found")
    assert exc.status_code == 404
    assert exc.detail == "Not found"

def test_http_exception_default_detail():
    """Test default detail messages."""
    exc = HTTPException(404)
    assert exc.detail == "Not Found"

    exc = HTTPException(500)
    assert exc.detail == "Internal Server Error"

def test_http_exception_to_response():
    """Test exception to response conversion."""
    exc = HTTPException(404, "Product not found")
    response = exc.to_response()

    assert response["status"] == 404
    assert response["body"]["error"] == "Product not found"
    assert response["body"]["status_code"] == 404

def test_validation_error():
    """Test ValidationError."""
    exc = ValidationError("Invalid data", errors={"field": "error"})
    assert exc.status_code == 422
    response = exc.to_response()
    assert response["body"]["errors"] == {"field": "error"}

def test_not_found_error():
    """Test NotFoundError."""
    exc = NotFoundError("User not found")
    assert exc.status_code == 404
    assert exc.detail == "User not found"

def test_app_error_handling():
    """Test App._handle_error method."""
    app = App(debug=False)

    # HTTPException
    exc = HTTPException(400, "Bad request")
    response = app._handle_error(exc)
    assert response["status"] == 400

    # Generic exception (production mode)
    exc = ValueError("Something broke")
    response = app._handle_error(exc)
    assert response["status"] == 500
    assert "traceback" not in response["body"]

def test_app_error_handling_debug():
    """Test App._handle_error in debug mode."""
    app = App(debug=True)

    # Generic exception (debug mode)
    exc = ValueError("Something broke")
    response = app._handle_error(exc)
    assert response["status"] == 500
    assert "traceback" in response["body"]
    assert "detail" in response["body"]

def test_duplicate_key_error():
    """Test duplicate key error detection."""
    app = App(debug=False)

    # Simulate MongoDB duplicate key error
    exc = Exception("E11000 duplicate key error")
    response = app._handle_error(exc)
    assert response["status"] == 409
    assert response["body"]["type"] == "ConflictError"

def test_validation_error_auto_detection():
    """Test automatic validation error detection."""
    app = App(debug=False)

    # Simulate validation error
    exc = ValueError("validation failed for field")
    response = app._handle_error(exc)
    assert response["status"] == 422
    assert response["body"]["type"] == "ValidationError"

def test_objectid_error():
    """Test ObjectId error detection."""
    app = App(debug=False)

    # Simulate ObjectId error
    exc = Exception("invalid objectid format")
    response = app._handle_error(exc)
    assert response["status"] == 400
    assert response["body"]["type"] == "BadRequest"

def test_http_exception_with_headers():
    """Test HTTPException with custom headers."""
    exc = HTTPException(401, "Unauthorized", headers={"WWW-Authenticate": "Bearer"})
    response = exc.to_response()
    assert response["headers"]["WWW-Authenticate"] == "Bearer"

def test_http_exception_with_extra():
    """Test HTTPException with extra data."""
    exc = HTTPException(400, "Bad request", extra={"field": "email", "reason": "invalid format"})
    response = exc.to_response()
    assert response["body"]["field"] == "email"
    assert response["body"]["reason"] == "invalid format"

@pytest.mark.asyncio
async def test_wrapped_handler_success():
    """Test that wrapped handler works for successful requests."""
    app = App(debug=False)

    async def test_handler(request):
        return {"status": 200, "body": {"message": "success"}}

    wrapped = app._wrap_handler_with_error_handling(test_handler)
    result = await wrapped({"path": "/test"})

    assert result["status"] == 200
    assert result["body"]["message"] == "success"

@pytest.mark.asyncio
async def test_wrapped_handler_http_exception():
    """Test that wrapped handler catches HTTPException."""
    app = App(debug=False)

    async def test_handler(request):
        raise NotFoundError("Resource not found")

    wrapped = app._wrap_handler_with_error_handling(test_handler)
    result = await wrapped({"path": "/test"})

    assert result["status"] == 404
    assert result["body"]["error"] == "Resource not found"

@pytest.mark.asyncio
async def test_wrapped_handler_generic_exception():
    """Test that wrapped handler catches generic exceptions."""
    app = App(debug=False)

    async def test_handler(request):
        raise ValueError("Something went wrong")

    wrapped = app._wrap_handler_with_error_handling(test_handler)
    result = await wrapped({"path": "/test"})

    assert result["status"] == 500
    assert result["body"]["error"] == "Internal Server Error"
    assert "traceback" not in result["body"]

@pytest.mark.asyncio
async def test_wrapped_handler_generic_exception_debug():
    """Test that wrapped handler includes traceback in debug mode."""
    app = App(debug=True)

    async def test_handler(request):
        raise ValueError("Something went wrong")

    wrapped = app._wrap_handler_with_error_handling(test_handler)
    result = await wrapped({"path": "/test"})

    assert result["status"] == 500
    assert result["body"]["error"] == "Internal Server Error"
    assert "traceback" in result["body"]
    assert "detail" in result["body"]

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
