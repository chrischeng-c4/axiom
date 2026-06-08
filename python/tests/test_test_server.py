"""Tests for TestServer with Python application support."""

import asyncio
import pytest
from cclab.probe import expect
import httpx
from cclab.probe import TestServer


@pytest.mark.asyncio
async def test_server_from_app_basic():
    """Test basic Python app startup and health check."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18765,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle = await server.start()

    try:
        # Verify server is running
        assert handle.url == "http://127.0.0.1:18765"
        assert handle.port == 18765

        # Test health endpoint
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/health")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "healthy"
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_server_from_app_api_endpoint():
    """Test accessing API endpoints."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18766,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle = await server.start()

    try:
        # Test API endpoint
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/api/users")
            assert response.status_code == 200
            data = response.json()
            assert "users" in data
            assert len(data["users"]) == 2
            assert data["users"][0]["name"] == "Alice"
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_server_from_app_echo_endpoint():
    """Test parameterized endpoint."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18767,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle = await server.start()

    try:
        # Test echo endpoint
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/api/echo/hello")
            assert response.status_code == 200
            data = response.json()
            assert data["message"] == "hello"
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_server_client_property():
    """Test that the client property returns the URL."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18768,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle = await server.start()

    try:
        # For now, client returns the URL string
        client = handle.client
        assert client == "http://127.0.0.1:18768"
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_server_shutdown():
    """Test graceful server shutdown."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18769,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle = await server.start()

    # Verify server is running
    async with httpx.AsyncClient() as client:
        response = await client.get(f"{handle.url}/health")
        assert response.status_code == 200

    # Stop the server
    handle.stop()

    # Give it a moment to shut down
    await asyncio.sleep(0.5)

    # Verify server is stopped (connection should be refused)
    expect(lambda: async with httpx.AsyncClient() as client:).to_raise(httpx.ConnectError)
            await client.get(f"{handle.url}/health", timeout=1.0)


@pytest.mark.asyncio
async def test_server_multiple_instances():
    """Test running multiple server instances simultaneously."""
    server1 = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18770,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    server2 = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18771,
        startup_timeout=10.0,
        health_endpoint="/health",
    )

    handle1 = await server1.start()
    handle2 = await server2.start()

    try:
        # Both servers should be running
        async with httpx.AsyncClient() as client:
            response1 = await client.get(f"{handle1.url}/health")
            assert response1.status_code == 200

            response2 = await client.get(f"{handle2.url}/health")
            assert response2.status_code == 200
    finally:
        handle1.stop()
        handle2.stop()


@pytest.mark.asyncio
async def test_server_no_health_endpoint():
    """Test server startup without explicit health check endpoint."""
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",
        app_callable="app",
        port=18772,
        startup_timeout=10.0,
        health_endpoint=None,  # No health endpoint
    )

    handle = await server.start()

    try:
        # Server should still start (uses TCP connection check)
        assert handle.url == "http://127.0.0.1:18772"

        # API should still work
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/api/users")
            assert response.status_code == 200
    finally:
        handle.stop()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
