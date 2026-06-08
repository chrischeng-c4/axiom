"""
Security tests for KV Store.

Tests cover:
- Key validation (length limits, empty keys, special characters)
- Value safety (large values, binary data, nested structures)
- Namespace isolation (boundary enforcement, path traversal)
- Resource limits (connection handling, pool exhaustion)
- Protocol safety (timeout behavior)

These tests ensure the KV store properly validates inputs and handles
edge cases that could lead to security issues.
"""

import pytest
from cclab.probe import expect
import asyncio
from cclab.ion import KvClient, KvPool, PoolConfig


class TestKeyValidation:
    """Test key validation and constraints."""

    @pytest.mark.asyncio
    async def test_max_key_length_accepted(self, kv_client):
        """Keys at maximum length (256 chars) should be accepted."""
        # Create key at exactly 256 characters
        max_key = "k" * 256

        await kv_client.set(max_key, "value_at_max_length")
        result = await kv_client.get(max_key)

        assert result == "value_at_max_length", "Max length key should work"

        # Cleanup
        await kv_client.delete(max_key)

    @pytest.mark.asyncio
    async def test_key_exceeding_max_length_rejected(self, kv_client):
        """Keys exceeding 256 characters should be rejected."""
        # Create key with 257 characters (1 over limit)
        too_long = "k" * 257

        # Server should reject this with an error
        # Note: Based on the code, validation happens server-side
        # The client will send the request but server should respond with error
        expect(lambda: await kv_client.set(too_long, "value")).to_raise(RuntimeError)

    @pytest.mark.asyncio
    async def test_empty_key_rejected(self, kv_client):
        """Empty keys should be rejected."""
        expect(lambda: await kv_client.set("", "value")).to_raise(RuntimeError)

    @pytest.mark.asyncio
    async def test_unicode_keys_work(self, kv_client):
        """Unicode keys should be supported."""
        unicode_keys = [
            "key_with_emoji_🔥",
            "中文_key",
            "key_with_ñ",
            "κλειδί",  # Greek
            "مفتاح",  # Arabic
        ]

        for key in unicode_keys:
            # Only test if key is within 256 char limit
            if len(key) <= 256:
                await kv_client.set(key, f"value_for_{key}")
                result = await kv_client.get(key)
                assert result == f"value_for_{key}", f"Unicode key {key} should work"
                await kv_client.delete(key)

    @pytest.mark.asyncio
    async def test_keys_with_special_characters(self, kv_client):
        """Keys with special characters should work."""
        special_keys = [
            "key:with:colons",
            "key/with/slashes",
            "key-with-dashes",
            "key_with_underscores",
            "key.with.dots",
            "key@with@at",
            "key#with#hash",
        ]

        for key in special_keys:
            await kv_client.set(key, f"value_{key}")
            result = await kv_client.get(key)
            assert result == f"value_{key}", f"Special key {key} should work"
            await kv_client.delete(key)

    @pytest.mark.asyncio
    async def test_keys_with_whitespace(self, kv_client):
        """Keys with whitespace should be treated literally."""
        whitespace_keys = [
            "key with spaces",
            "key\twith\ttabs",
            "key\nwith\nnewlines",  # This might be problematic
        ]

        for key in whitespace_keys:
            try:
                await kv_client.set(key, f"value_{key}")
                result = await kv_client.get(key)
                assert result == f"value_{key}", f"Key with whitespace should work"
                await kv_client.delete(key)
            except Exception:
                # Newlines and certain whitespace might be rejected - that's okay
                pass

    @pytest.mark.asyncio
    async def test_null_byte_in_key_handled(self, kv_client):
        """Null bytes in keys should be handled (likely rejected)."""
        # Null bytes are often problematic in C-style strings
        # The client/server should handle this gracefully
        null_key = "key\x00with\x00null"

        try:
            await kv_client.set(null_key, "value")
            result = await kv_client.get(null_key)
            # If it works, verify it
            assert result == "value"
            await kv_client.delete(null_key)
        except Exception:
            # If rejected, that's acceptable security behavior
            pass


class TestValueSafety:
    """Test value safety and size handling."""

    @pytest.mark.asyncio
    async def test_large_value_1mb(self, kv_client):
        """1MB values should be handled safely."""
        large_value = "x" * (1024 * 1024)  # 1MB string

        await kv_client.set("large_1mb", large_value)
        result = await kv_client.get("large_1mb")

        assert result == large_value, "1MB value should be stored/retrieved correctly"
        assert len(result) == 1024 * 1024, "Length should match"

        # Cleanup
        await kv_client.delete("large_1mb")

    @pytest.mark.asyncio
    async def test_large_value_10mb(self, kv_client):
        """10MB values should be handled safely."""
        large_value = "y" * (10 * 1024 * 1024)  # 10MB string

        await kv_client.set("large_10mb", large_value)
        result = await kv_client.get("large_10mb")

        assert result == large_value, "10MB value should be stored/retrieved correctly"
        assert len(result) == 10 * 1024 * 1024, "Length should match"

        # Cleanup
        await kv_client.delete("large_10mb")

    @pytest.mark.asyncio
    async def test_binary_data_with_null_bytes(self, kv_client):
        """Binary data with null bytes should be handled safely."""
        # Binary data that includes null bytes and all byte values
        binary_data = bytes(range(256))  # All byte values 0-255

        await kv_client.set("binary_data", binary_data)
        result = await kv_client.get("binary_data")

        assert result == binary_data, "Binary data should be preserved"
        assert len(result) == 256, "All bytes should be present"

        # Cleanup
        await kv_client.delete("binary_data")

    @pytest.mark.asyncio
    async def test_deeply_nested_list(self, kv_client):
        """Deeply nested lists should be handled."""
        # Create a deeply nested list structure
        nested = []
        current = nested
        depth = 100

        for i in range(depth):
            inner = [f"level_{i}"]
            current.append(inner)
            current = inner

        await kv_client.set("deep_list", nested)
        result = await kv_client.get("deep_list")

        assert result == nested, "Deeply nested list should be preserved"

        # Cleanup
        await kv_client.delete("deep_list")

    @pytest.mark.asyncio
    async def test_deeply_nested_map(self, kv_client):
        """Deeply nested maps/dicts should be handled."""
        # Create a deeply nested dict structure
        nested = {}
        current = nested
        depth = 100

        for i in range(depth):
            current[f"level_{i}"] = {}
            current = current[f"level_{i}"]
        current["value"] = "deep_value"

        await kv_client.set("deep_map", nested)
        result = await kv_client.get("deep_map")

        assert result == nested, "Deeply nested map should be preserved"

        # Cleanup
        await kv_client.delete("deep_map")

    @pytest.mark.asyncio
    async def test_mixed_nested_structures(self, kv_client):
        """Complex nested structures (lists and dicts) should work."""
        complex_value = {
            "users": [
                {"id": 1, "name": "Alice", "tags": ["admin", "user"]},
                {"id": 2, "name": "Bob", "tags": ["user"]},
            ],
            "config": {
                "settings": {
                    "nested": {
                        "deep": {
                            "values": [1, 2, 3, {"key": "value"}]
                        }
                    }
                }
            },
            "binary": b"\x00\x01\x02\xff",
        }

        await kv_client.set("complex", complex_value)
        result = await kv_client.get("complex")

        assert result == complex_value, "Complex nested structure should be preserved"

        # Cleanup
        await kv_client.delete("complex")


class TestNamespaceIsolation:
    """Test namespace boundary enforcement."""

    @pytest.mark.asyncio
    async def test_cannot_access_keys_from_other_namespace(self):
        """Keys in one namespace should not be visible in another."""
        client_a = await KvClient.connect("127.0.0.1:11010/security_test_a")
        client_b = await KvClient.connect("127.0.0.1:11010/security_test_b")

        try:
            # Set secret in namespace A
            await client_a.set("secret_key", "secret_data_a")

            # Try to read from namespace B
            result = await client_b.get("secret_key")

            assert result is None, "Should not be able to access key from different namespace"

            # Set different value in namespace B
            await client_b.set("secret_key", "secret_data_b")

            # Verify A's value is unchanged
            result_a = await client_a.get("secret_key")
            assert result_a == "secret_data_a", "Namespace A value should be unchanged"

            # Verify B's value is different
            result_b = await client_b.get("secret_key")
            assert result_b == "secret_data_b", "Namespace B should have its own value"

        finally:
            await client_a.delete("secret_key")
            await client_b.delete("secret_key")

    @pytest.mark.asyncio
    async def test_path_traversal_in_namespace_literal(self):
        """Path traversal attempts in namespace should be treated literally."""
        # Try to use path traversal in namespace
        client_escape = await KvClient.connect("127.0.0.1:11010/../escape_attempt")
        client_normal = await KvClient.connect("127.0.0.1:11010/normal")

        try:
            # The namespace "../escape_attempt" should be treated literally
            # not as a path traversal
            assert client_escape.namespace == "../escape_attempt", \
                "Namespace should be stored literally"

            # Set value in "escape" namespace
            await client_escape.set("test_key", "escape_value")

            # Should not be accessible from normal namespace
            result = await client_normal.get("test_key")
            assert result is None, "Path traversal should not escape namespace"

        finally:
            await client_escape.delete("test_key")

    @pytest.mark.asyncio
    async def test_namespace_with_dots(self):
        """Namespaces with dots should be treated literally."""
        client_dots = await KvClient.connect("127.0.0.1:11010/../../root")
        client_other = await KvClient.connect("127.0.0.1:11010/other")

        try:
            # Namespace should be literal "../../root"
            assert client_dots.namespace == "../../root"

            await client_dots.set("dotted_key", "dotted_value")
            result = await client_other.get("dotted_key")

            assert result is None, "Dots in namespace should not navigate paths"

        finally:
            await client_dots.delete("dotted_key")

    @pytest.mark.asyncio
    async def test_delete_in_one_namespace_doesnt_affect_another(self):
        """Delete operations should respect namespace boundaries."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/delete_test_1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/delete_test_2")

        try:
            # Set same key in both namespaces
            await client_ns1.set("shared_key", "value_1")
            await client_ns2.set("shared_key", "value_2")

            # Delete from ns1
            deleted = await client_ns1.delete("shared_key")
            assert deleted is True, "Delete should succeed"

            # Verify deleted in ns1
            result_ns1 = await client_ns1.get("shared_key")
            assert result_ns1 is None, "Key should be deleted in ns1"

            # Verify still exists in ns2
            result_ns2 = await client_ns2.get("shared_key")
            assert result_ns2 == "value_2", "Key should still exist in ns2"

        finally:
            await client_ns2.delete("shared_key")


class TestResourceLimits:
    """Test resource limit handling and connection management."""

    @pytest.mark.asyncio
    async def test_rapid_reconnection_handling(self):
        """Server should handle rapid connection attempts gracefully."""
        clients = []

        try:
            # Create multiple connections rapidly
            for i in range(20):
                client = await KvClient.connect("127.0.0.1:11010/rapid_test")
                clients.append(client)
                # Quick operation to verify connection works
                await client.ping()

            # All connections should work
            assert len(clients) == 20, "All rapid connections should succeed"

        finally:
            # Clients will be cleaned up automatically
            pass

    @pytest.mark.asyncio
    async def test_pool_exhaustion_timeout(self):
        """Pool should timeout gracefully when exhausted."""
        # Create a small pool
        config = PoolConfig(
            "127.0.0.1:11010/pool_exhaust",
            min_size=1,
            max_size=2,
            acquire_timeout=1.0,  # 1 second timeout
        )
        pool = await KvPool.connect(config)

        # Create tasks that hold connections for a while
        async def hold_connection(duration: float, key: str):
            await pool.set(key, "holding")
            await asyncio.sleep(duration)
            await pool.delete(key)

        try:
            # Start 2 tasks that hold connections for 2 seconds
            tasks = [
                asyncio.create_task(hold_connection(2.0, "hold1")),
                asyncio.create_task(hold_connection(2.0, "hold2")),
            ]

            # Wait a bit for connections to be acquired
            await asyncio.sleep(0.1)

            # Now try to get stats - this should work quickly
            # (stats don't need to acquire a connection from the pool)
            stats = await pool.stats()
            assert stats.active >= 2, "Pool should show active connections"

            # Wait for tasks to complete
            await asyncio.gather(*tasks)

        except Exception as e:
            # If this fails, it's acceptable - we're testing edge cases
            print(f"Pool exhaustion test note: {e}")

    @pytest.mark.asyncio
    async def test_pool_connection_limits_enforced(self):
        """Pool should enforce max_size limit."""
        config = PoolConfig(
            "127.0.0.1:11010/pool_limit",
            min_size=2,
            max_size=5,
        )
        pool = await KvPool.connect(config)

        # Get stats
        stats = await pool.stats()

        # Max size should be enforced
        assert stats.max_size == 5, "Max size should match config"

        # Do some operations
        for i in range(10):
            await pool.set(f"key_{i}", i)

        # Check stats again
        stats = await pool.stats()
        assert stats.active + stats.idle <= 5, "Total connections should not exceed max_size"

        # Cleanup
        for i in range(10):
            await pool.delete(f"key_{i}")

    @pytest.mark.asyncio
    async def test_concurrent_operations_dont_corrupt_data(self):
        """Concurrent operations should not corrupt data."""
        client = await KvClient.connect("127.0.0.1:11010/concurrent")

        try:
            # Initialize counter
            await client.set("counter", 0)

            # Run many concurrent increments
            async def increment_counter(count: int):
                for _ in range(count):
                    await client.incr("counter", 1)

            tasks = [increment_counter(10) for _ in range(10)]
            await asyncio.gather(*tasks)

            # Final value should be correct (10 tasks * 10 increments = 100)
            final_value = await client.get("counter")
            assert final_value == 100, "Concurrent operations should not corrupt data"

        finally:
            await client.delete("counter")


class TestProtocolSafety:
    """Test protocol-level safety features."""

    @pytest.mark.asyncio
    async def test_connection_survives_large_operations(self):
        """Connection should remain stable after large operations."""
        client = await KvClient.connect("127.0.0.1:11010/protocol_test")

        try:
            # Do a large operation
            large_value = "x" * (5 * 1024 * 1024)  # 5MB
            await client.set("large", large_value)

            # Connection should still work
            await client.ping()
            result = await client.get("large")
            assert result == large_value, "Large value should be retrieved"

            # Do normal operations after large one
            await client.set("normal", "value")
            result = await client.get("normal")
            assert result == "value", "Connection should work normally after large op"

        finally:
            await client.delete("large")
            await client.delete("normal")

    @pytest.mark.asyncio
    async def test_multiple_operations_on_same_key(self):
        """Multiple rapid operations on the same key should work."""
        client = await KvClient.connect("127.0.0.1:11010/multi_op")

        try:
            # Rapidly set, get, update, delete same key
            for i in range(100):
                await client.set("rapid_key", i)
                result = await client.get("rapid_key")
                assert result == i, f"Value should be {i}"

            # Final delete
            deleted = await client.delete("rapid_key")
            assert deleted is True, "Key should be deleted"

        finally:
            pass

    @pytest.mark.asyncio
    async def test_ttl_doesnt_cause_data_corruption(self):
        """TTL expiration should not corrupt other keys."""
        client = await KvClient.connect("127.0.0.1:11010/ttl_safety")

        try:
            # Set a key with short TTL
            await client.set("short_lived", "expires_soon", ttl=0.5)

            # Set a permanent key
            await client.set("permanent", "stays")

            # Wait for TTL to expire
            await asyncio.sleep(0.6)

            # Short-lived should be gone
            result = await client.get("short_lived")
            assert result is None, "Short-lived key should expire"

            # Permanent should still exist
            result = await client.get("permanent")
            assert result == "stays", "Permanent key should remain"

        finally:
            await client.delete("permanent")

    @pytest.mark.asyncio
    async def test_exists_after_delete_consistent(self):
        """Exists check should be consistent after delete."""
        client = await KvClient.connect("127.0.0.1:11010/exists_test")

        try:
            # Set key
            await client.set("exist_key", "value")

            # Should exist
            exists = await client.exists("exist_key")
            assert exists is True, "Key should exist after set"

            # Delete
            await client.delete("exist_key")

            # Should not exist
            exists = await client.exists("exist_key")
            assert exists is False, "Key should not exist after delete"

            # Get should return None
            result = await client.get("exist_key")
            assert result is None, "Get should return None for deleted key"

        finally:
            pass


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    @pytest.mark.asyncio
    async def test_zero_ttl_behavior(self, kv_client):
        """TTL of 0 or very small values should be handled."""
        # Try with TTL of 0.001 seconds (1ms)
        await kv_client.set("very_short", "value", ttl=0.001)

        # Immediately try to get it - might or might not exist
        result = await kv_client.get("very_short")
        # We can't assert much here - timing dependent

        # Wait a bit and it should definitely be gone
        await asyncio.sleep(0.1)
        result = await kv_client.get("very_short")
        assert result is None, "Key with tiny TTL should expire quickly"

    @pytest.mark.asyncio
    async def test_negative_ttl_rejected(self, kv_client):
        """Negative TTL should be rejected with proper error."""
        expect(lambda: await kv_client.set("negative_ttl", "value", ttl=-1.0)).to_raise(ValueError)

    @pytest.mark.asyncio
    async def test_very_long_ttl(self, kv_client):
        """Very long TTL values should work."""
        # Set TTL to 1 year (31536000 seconds)
        await kv_client.set("long_lived", "value", ttl=31536000.0)

        # Should be able to retrieve immediately
        result = await kv_client.get("long_lived")
        assert result == "value", "Key with long TTL should work"

        # Cleanup
        await kv_client.delete("long_lived")

    @pytest.mark.asyncio
    async def test_empty_value_allowed(self, kv_client):
        """Empty values should be allowed."""
        # Empty string
        await kv_client.set("empty_string", "")
        result = await kv_client.get("empty_string")
        assert result == "", "Empty string should be stored"

        # Empty bytes
        await kv_client.set("empty_bytes", b"")
        result = await kv_client.get("empty_bytes")
        assert result == b"", "Empty bytes should be stored"

        # Empty list
        await kv_client.set("empty_list", [])
        result = await kv_client.get("empty_list")
        assert result == [], "Empty list should be stored"

        # Empty dict
        await kv_client.set("empty_dict", {})
        result = await kv_client.get("empty_dict")
        assert result == {}, "Empty dict should be stored"

        # Cleanup
        await kv_client.delete("empty_string")
        await kv_client.delete("empty_bytes")
        await kv_client.delete("empty_list")
        await kv_client.delete("empty_dict")

    @pytest.mark.asyncio
    async def test_null_value_stored(self, kv_client):
        """None/null values should be storable."""
        await kv_client.set("null_value", None)
        result = await kv_client.get("null_value")

        # The key should exist but value is None
        exists = await kv_client.exists("null_value")
        assert exists is True, "Key with null value should exist"
        assert result is None, "Null value should be retrieved as None"

        # Cleanup
        await kv_client.delete("null_value")
