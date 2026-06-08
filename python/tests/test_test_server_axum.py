"""Tests for TestServer with Axum static routes (existing functionality)."""

import pytest
import httpx
from cclab.probe import TestServer


@pytest.mark.asyncio
async def test_axum_server_basic():
    """Test basic Axum server with static routes."""
    server = TestServer()
    server.get("/test", {"status": "ok", "message": "hello"})
    server.port(19000)

    handle = await server.start()

    try:
        assert handle.port == 19000
        assert handle.url == "http://127.0.0.1:19000"

        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/test")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "ok"
            assert data["message"] == "hello"
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_axum_server_auto_port():
    """Test Axum server with auto-selected port."""
    server = TestServer()
    server.get("/ping", {"pong": True})

    handle = await server.start()

    try:
        # Port should be auto-selected
        assert handle.port > 0

        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/ping")
            assert response.status_code == 200
            data = response.json()
            assert data["pong"] is True
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_axum_server_multiple_routes():
    """Test Axum server with multiple routes."""
    server = TestServer()
    server.get("/users", {"users": ["alice", "bob"]})
    server.get("/posts", {"posts": ["post1", "post2"]})
    server.port(19001)

    handle = await server.start()

    try:
        async with httpx.AsyncClient() as client:
            # Test first route
            response1 = await client.get(f"{handle.url}/users")
            assert response1.status_code == 200
            assert response1.json()["users"] == ["alice", "bob"]

            # Test second route
            response2 = await client.get(f"{handle.url}/posts")
            assert response2.status_code == 200
            assert response2.json()["posts"] == ["post1", "post2"]
    finally:
        handle.stop()


@pytest.mark.asyncio
async def test_axum_server_404():
    """Test Axum server 404 for unknown routes."""
    server = TestServer()
    server.get("/exists", {"status": "ok"})
    server.port(19002)

    handle = await server.start()

    try:
        async with httpx.AsyncClient() as client:
            # Known route works
            response = await client.get(f"{handle.url}/exists")
            assert response.status_code == 200

            # Unknown route returns 404
            response = await client.get(f"{handle.url}/unknown")
            assert response.status_code == 404
    finally:
        handle.stop()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
