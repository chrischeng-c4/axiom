"""Example: Using TestServer to auto-start Python applications for testing.

This example demonstrates how to use TestServer.from_app() to automatically
start a Python web application (Flask, FastAPI, etc.) as a subprocess and
test it using standard HTTP clients.
"""

import asyncio
import httpx
from cclab.probe import TestServer


async def main():
    """Demonstrate TestServer.from_app() functionality."""
    print("=" * 60)
    print("TestServer.from_app() Example")
    print("=" * 60)

    # Create a TestServer that will spawn a Flask application
    print("\n1. Creating TestServer from Python app...")
    server = TestServer.from_app(
        app_module="tests.fixtures.test_app",  # Python module to import
        app_callable="app",  # Name of the Flask app instance
        port=18800,  # Port to bind to
        startup_timeout=10.0,  # Max seconds to wait for startup
        health_endpoint="/health",  # Endpoint to check for readiness
    )

    # Start the server (spawns subprocess and waits for health check)
    print("2. Starting server (spawning subprocess)...")
    handle = await server.start()
    print(f"   ✓ Server started at {handle.url}")

    try:
        # Test the health endpoint
        print("\n3. Testing health endpoint...")
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/health")
            print(f"   Status: {response.status_code}")
            print(f"   Response: {response.json()}")

        # Test an API endpoint
        print("\n4. Testing API endpoint...")
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/api/users")
            print(f"   Status: {response.status_code}")
            data = response.json()
            print(f"   Users: {len(data['users'])} found")
            for user in data["users"]:
                print(f"     - {user['name']} (ID: {user['id']})")

        # Test a parameterized endpoint
        print("\n5. Testing parameterized endpoint...")
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{handle.url}/api/echo/hello-world")
            print(f"   Status: {response.status_code}")
            print(f"   Echo: {response.json()['message']}")

        print("\n" + "=" * 60)
        print("All tests completed successfully!")
        print("=" * 60)

    finally:
        # Stop the server (kills subprocess gracefully)
        print("\n6. Stopping server...")
        handle.stop()
        print("   ✓ Server stopped")


if __name__ == "__main__":
    asyncio.run(main())
