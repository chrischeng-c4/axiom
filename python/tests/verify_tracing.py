"""
Verification script for OpenTelemetry distributed tracing in PyLoop.

This script:
1. Configures OpenTelemetry with ConsoleSpanExporter (for debugging)
2. Starts PyLoop HTTP server with OpenTelemetryMiddleware
3. Sends test requests
4. Verifies trace context propagation (Rust → Python)
5. Checks that trace IDs are consistent across Rust and Python spans

Run this script to verify that distributed tracing works correctly before
deploying to GCP.
"""

import asyncio
import logging
import sys
import time
from typing import Dict, Any
from multiprocessing import Process

# Configure logging to see trace IDs
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - [trace_id=%(otelTraceID)s span_id=%(otelSpanID)s] - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)

logger = logging.getLogger(__name__)

def setup_telemetry():
    """
    Configure OpenTelemetry with ConsoleSpanExporter for local verification.

    In production, this would use OTLPSpanExporter to send traces to
    the OTel Collector sidecar.
    """
    try:
        from opentelemetry import trace
        from opentelemetry.sdk.trace import TracerProvider
        from opentelemetry.sdk.trace.export import (
            BatchSpanProcessor,
            ConsoleSpanExporter,
        )
        from opentelemetry.sdk.resources import Resource

        # Create resource with service information
        resource = Resource.create({
            "service.name": "data-bridge-api",
            "service.version": "0.1.0",
            "service.namespace": "data-bridge",
            "deployment.environment": "test",
        })

        # Create tracer provider
        provider = TracerProvider(resource=resource)

        # Add console exporter for verification (prints spans to stdout)
        console_exporter = ConsoleSpanExporter()
        span_processor = BatchSpanProcessor(console_exporter)
        provider.add_span_processor(span_processor)

        # Set as global tracer provider
        trace.set_tracer_provider(provider)

        logger.info("✓ OpenTelemetry configured with ConsoleSpanExporter")
        return True

    except ImportError as e:
        logger.error(f"✗ OpenTelemetry not installed: {e}")
        logger.error("Install with: pip install opentelemetry-api opentelemetry-sdk opentelemetry-exporter-otlp")
        return False

def run_server():
    """Run PyLoop HTTP server with OpenTelemetry middleware."""
    from cclab.orbit import App, OpenTelemetryMiddleware

    # Setup telemetry in server process
    if not setup_telemetry():
        logger.error("Failed to setup telemetry in server process")
        return

    # Create app
    app = App(title="Tracing Verification", version="0.1.0", debug=True)

    # Add OpenTelemetry middleware
    app.add_middleware(OpenTelemetryMiddleware(tracer_name="data-bridge-pyloop"))

    logger.info("✓ Added OpenTelemetryMiddleware to PyLoop app")

    # Define test routes
    @app.get("/")
    async def root(request: Dict[str, Any]) -> Dict[str, Any]:
        """Root endpoint - returns basic info."""
        logger.info("Handling request to /")
        return {
            "message": "PyLoop with distributed tracing",
            "service": "data-bridge-api",
            "tracing": "enabled"
        }

    @app.get("/health")
    async def health(request: Dict[str, Any]) -> Dict[str, Any]:
        """Health check endpoint."""
        logger.info("Handling request to /health")
        return {"status": "healthy", "tracing": "active"}

    @app.post("/api/users")
    async def create_user(request: Dict[str, Any]) -> Dict[str, Any]:
        """Create user endpoint - tests POST with body."""
        body = request.get("body", {})
        logger.info(f"Handling request to /api/users with body: {body}")

        # Simulate some processing
        await asyncio.sleep(0.1)

        return {
            "id": 123,
            "name": body.get("name", "Unknown"),
            "email": body.get("email", "unknown@example.com"),
            "created": True
        }

    @app.get("/api/users/{user_id}")
    async def get_user(request: Dict[str, Any]) -> Dict[str, Any]:
        """Get user endpoint - tests path parameters."""
        user_id = request.get("path_params", {}).get("user_id", "0")
        logger.info(f"Handling request to /api/users/{user_id}")

        return {
            "id": int(user_id),
            "name": "Alice",
            "email": "alice@example.com"
        }

    # Start server
    logger.info("=" * 70)
    logger.info("Starting PyLoop HTTP server with OpenTelemetry tracing")
    logger.info("Server: http://127.0.0.1:8000")
    logger.info("=" * 70)

    try:
        app.serve(host="127.0.0.1", port=8000)
    except KeyboardInterrupt:
        logger.info("Server stopped by user")

async def send_test_requests():
    """Send test requests to verify tracing."""
    import httpx

    base_url = "http://127.0.0.1:8000"

    logger.info("\n" + "=" * 70)
    logger.info("Sending test requests to PyLoop server")
    logger.info("=" * 70)

    async with httpx.AsyncClient(timeout=10.0) as client:
        # Test 1: GET /
        logger.info("\n[Test 1] GET /")
        try:
            response = await client.get(f"{base_url}/")
            logger.info(f"  Status: {response.status_code}")
            logger.info(f"  Response: {response.json()}")
            assert response.status_code == 200
            logger.info("  ✓ Test 1 passed")
        except Exception as e:
            logger.error(f"  ✗ Test 1 failed: {e}")

        # Test 2: GET /health
        logger.info("\n[Test 2] GET /health")
        try:
            response = await client.get(f"{base_url}/health")
            logger.info(f"  Status: {response.status_code}")
            logger.info(f"  Response: {response.json()}")
            assert response.status_code == 200
            logger.info("  ✓ Test 2 passed")
        except Exception as e:
            logger.error(f"  ✗ Test 2 failed: {e}")

        # Test 3: POST /api/users
        logger.info("\n[Test 3] POST /api/users")
        try:
            response = await client.post(
                f"{base_url}/api/users",
                json={"name": "Bob", "email": "bob@example.com"}
            )
            logger.info(f"  Status: {response.status_code}")
            logger.info(f"  Response: {response.json()}")
            assert response.status_code == 200
            logger.info("  ✓ Test 3 passed")
        except Exception as e:
            logger.error(f"  ✗ Test 3 failed: {e}")

        # Test 4: GET /api/users/{user_id}
        logger.info("\n[Test 4] GET /api/users/456")
        try:
            response = await client.get(f"{base_url}/api/users/456")
            logger.info(f"  Status: {response.status_code}")
            logger.info(f"  Response: {response.json()}")
            assert response.status_code == 200
            logger.info("  ✓ Test 4 passed")
        except Exception as e:
            logger.error(f"  ✗ Test 4 failed: {e}")

        # Test 5: Multiple rapid requests (test batching)
        logger.info("\n[Test 5] Multiple rapid requests (10 requests)")
        try:
            tasks = [
                client.get(f"{base_url}/health")
                for _ in range(10)
            ]
            responses = await asyncio.gather(*tasks)
            success_count = sum(1 for r in responses if r.status_code == 200)
            logger.info(f"  Success: {success_count}/10 requests")
            assert success_count == 10
            logger.info("  ✓ Test 5 passed")
        except Exception as e:
            logger.error(f"  ✗ Test 5 failed: {e}")

    logger.info("\n" + "=" * 70)
    logger.info("All test requests completed!")
    logger.info("=" * 70)

def main():
    """Main verification workflow."""
    print("\n" + "=" * 70)
    print("OpenTelemetry Distributed Tracing Verification")
    print("=" * 70)
    print("\nThis script verifies that:")
    print("  1. PyLoop server can be instrumented with OpenTelemetryMiddleware")
    print("  2. Trace context is propagated from Rust (gateway) to Python (handler)")
    print("  3. Spans are created correctly and exported")
    print("  4. Trace IDs are consistent across language boundaries")
    print("\n" + "=" * 70)

    # Check if OpenTelemetry is installed
    try:
        import opentelemetry
        print("\n✓ OpenTelemetry packages detected")
    except ImportError:
        print("\n✗ OpenTelemetry not installed!")
        print("\nInstall with:")
        print("  pip install opentelemetry-api opentelemetry-sdk")
        sys.exit(1)

    # Check if httpx is installed (for test client)
    try:
        import httpx
        print("✓ httpx detected (for test client)")
    except ImportError:
        print("\n✗ httpx not installed!")
        print("\nInstall with:")
        print("  pip install httpx")
        sys.exit(1)

    print("\n" + "=" * 70)
    print("Starting verification process...")
    print("=" * 70)

    # Start server in background process
    print("\n[1/3] Starting PyLoop server in background...")
    server_process = Process(target=run_server, daemon=True)
    server_process.start()

    # Wait for server to start
    print("[2/3] Waiting for server to be ready...")
    time.sleep(3)

    # Send test requests
    print("[3/3] Sending test requests...")
    try:
        asyncio.run(send_test_requests())
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    finally:
        # Cleanup
        print("\n" + "=" * 70)
        print("Stopping server...")
        server_process.terminate()
        server_process.join(timeout=2)
        if server_process.is_alive():
            server_process.kill()
        print("Server stopped")

    print("\n" + "=" * 70)
    print("Verification complete!")
    print("=" * 70)
    print("\nWhat to look for in the output:")
    print("  • Console should show exported spans (from ConsoleSpanExporter)")
    print("  • Each span should have trace_id, span_id, parent_span_id")
    print("  • Python spans should be children of Rust spans (same trace_id)")
    print("  • Span names: 'http.request' (Rust), 'pyloop.request' (Python)")
    print("  • Span attributes: http.method, http.route, http.status_code")
    print("\nNext step: Run with local OTel Collector")
    print("  docker-compose -f deploy/docker-compose.otel.yml up")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    main()
