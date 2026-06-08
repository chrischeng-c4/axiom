"""
Pytest fixtures for KV store tests.

These fixtures provide:
- Session-scoped kv-server startup
- Function-scoped client connection
- Cleanup between tests
"""

import pytest
import asyncio
import subprocess
import time
import socket
from cclab.ion import KvClient


# KV server configuration for tests
KV_SERVER_HOST = "127.0.0.1"
KV_SERVER_PORT = 11010
KV_SERVER_ADDR = f"{KV_SERVER_HOST}:{KV_SERVER_PORT}"


def is_port_open(host: str, port: int, timeout: float = 1.0) -> bool:
    """Check if a port is open."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        result = sock.connect_ex((host, port))
        return result == 0
    finally:
        sock.close()


@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for async tests."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(scope="session", autouse=True)
def kv_server():
    """
    Start kv-server for the test session.

    This fixture runs once per test session and starts the KV server
    on port 11010.
    """
    # Check if server is already running
    if is_port_open(KV_SERVER_HOST, KV_SERVER_PORT):
        print(f"\nKV server already running on {KV_SERVER_ADDR}")
        yield
        return

    # Start kv-server process
    print(f"\nStarting kv-server on {KV_SERVER_ADDR}...")
    process = subprocess.Popen(
        [
            "./target/release/kv-server",
            "--bind",
            KV_SERVER_ADDR,
            "--shards",
            "256",
            "--log-level",
            "info",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Wait for server to start (max 5 seconds)
    for i in range(50):
        if is_port_open(KV_SERVER_HOST, KV_SERVER_PORT, timeout=0.1):
            print(f"KV server started successfully on {KV_SERVER_ADDR}")
            break
        time.sleep(0.1)
    else:
        # Server didn't start, kill process and raise error
        process.terminate()
        process.wait(timeout=2)
        stdout, stderr = process.communicate()
        raise RuntimeError(
            f"KV server failed to start on {KV_SERVER_ADDR}\n"
            f"STDOUT: {stdout.decode()}\n"
            f"STDERR: {stderr.decode()}"
        )

    yield

    # Cleanup - terminate server
    print(f"\nStopping kv-server...")
    process.terminate()
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


@pytest.fixture
async def kv_client(kv_server):
    """
    Create a KV client for each test.

    This fixture automatically connects to the test server and
    provides a clean client for each test.
    """
    client = await KvClient.connect(KV_SERVER_ADDR)
    yield client
    # Client cleanup handled by Python/Rust


@pytest.fixture(autouse=True)
async def cleanup_keys(kv_client):
    """
    Clean up test keys after each test.

    This ensures test isolation by removing all keys that might
    have been created during the test.
    """
    # List of common test keys to clean up
    test_keys = [
        "test_key",
        "test_setnx",
        "test_setnx_existing",
        "test_setnx_ttl",
        "test_lock",
        "test_lock_reentrant",
        "test_lock_different",
        "lock:resource",
        "lock:resource:1",
        "lock:resource:2",
        "lock:task",
        "lock:extend",
        "lock:nested:1",
        "lock:nested:2",
        # Benchmark test keys
        "warmup:pool",
        "bench:counter",
    ]

    # Key prefixes to clean up (for bulk operations)
    key_prefixes = [
        "warmup:set:",
        "bench:set:",
        "bench:get:",
        "bench:mixed:",
        "pool:set:",
        "pool:mixed:",
        "single:",
        "pool:",
        "cmp:db:",
        "cmp:redis:",
        "latency:set:",
        "latency:get:",
    ]

    yield

    # Cleanup after test - single keys
    for key in test_keys:
        try:
            await kv_client.delete(key)
        except Exception:
            pass  # Ignore errors during cleanup

    # Note: Bulk prefix cleanup would require a KEYS/SCAN command
    # which is not yet implemented in the KV server.
    # For now, benchmark tests handle their own cleanup.
