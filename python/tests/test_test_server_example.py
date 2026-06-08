"""Example demonstrating TestServer integration with fixture system."""

import asyncio
import pytest
import httpx
from cclab.probe import TestSuite, test, fixture, TestServer, expect


class APITests(TestSuite):
    """Example test suite using TestServer with fixture."""

    @fixture(scope="class")
    async def server(self):
        """Auto-start server from Python app."""
        server = TestServer.from_app(
            app_module="tests.fixtures.test_app",
            app_callable="app",
            port=18780,
            startup_timeout=10.0,
            health_endpoint="/health",
        )
        handle = await server.start()
        yield handle
        handle.stop()

    @test
    async def test_health_endpoint(self, server):
        """Test health check endpoint."""
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{server.url}/health")
            expect(response.status_code).to_equal(200)
            data = response.json()
            expect(data["status"]).to_equal("healthy")

    @test
    async def test_users_endpoint(self, server):
        """Test users endpoint."""
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{server.url}/api/users")
            expect(response.status_code).to_equal(200)
            data = response.json()
            expect("users" in data).to_be_true()
            expect(len(data["users"])).to_equal(2)

    @test
    async def test_echo_endpoint(self, server):
        """Test parameterized echo endpoint."""
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{server.url}/api/echo/test-message")
            expect(response.status_code).to_equal(200)
            data = response.json()
            expect(data["message"]).to_equal("test-message")


# Run with pytest
if __name__ == "__main__":
    pytest.main([__file__, "-v"])
