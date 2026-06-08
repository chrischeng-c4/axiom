"""
Integration tests for KV Store Namespace feature.

Tests namespace functionality including:
- Namespace property access
- Key isolation between namespaces
- Lock isolation between namespaces
- Namespace syntax (simple and nested)
"""

import pytest
import asyncio
from cclab.ion import KvClient, Lock


class TestNamespaceProperty:
    """Test namespace property access."""

    @pytest.mark.asyncio
    async def test_no_namespace_returns_none(self):
        """Test client without namespace returns None."""
        client = await KvClient.connect("127.0.0.1:11010")

        assert client.namespace is None, "Client without namespace should return None"

    @pytest.mark.asyncio
    async def test_simple_namespace_returns_correct_value(self):
        """Test client with simple namespace returns correct value."""
        client = await KvClient.connect("127.0.0.1:11010/tasks")

        assert client.namespace == "tasks", "Client should return correct namespace"

    @pytest.mark.asyncio
    async def test_nested_namespace_returns_correct_value(self):
        """Test client with nested namespace returns correct value."""
        client = await KvClient.connect("127.0.0.1:11010/prod/cache")

        assert client.namespace == "prod/cache", "Client should return correct nested namespace"


class TestKeyIsolation:
    """Test key isolation between namespaces."""

    @pytest.mark.asyncio
    async def test_keys_in_different_namespaces_are_isolated(self):
        """Test keys in different namespaces don't interfere."""
        # Create clients for different namespaces
        client_tasks = await KvClient.connect("127.0.0.1:11010/tasks")
        client_cache = await KvClient.connect("127.0.0.1:11010/cache")

        try:
            # Set same key in both namespaces
            await client_tasks.set("shared_key", "tasks_value")
            await client_cache.set("shared_key", "cache_value")

            # Values should be isolated
            tasks_value = await client_tasks.get("shared_key")
            cache_value = await client_cache.get("shared_key")

            assert tasks_value == "tasks_value", "Tasks namespace should have its own value"
            assert cache_value == "cache_value", "Cache namespace should have its own value"
        finally:
            # Cleanup
            await client_tasks.delete("shared_key")
            await client_cache.delete("shared_key")

    @pytest.mark.asyncio
    async def test_same_key_different_namespaces_different_values(self):
        """Test same key name can have different values in different namespaces."""
        # Create clients for different namespaces
        client_ns1 = await KvClient.connect("127.0.0.1:11010/namespace1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/namespace2")

        try:
            # Set different types/values with same key name
            await client_ns1.set("counter", 100)
            await client_ns2.set("counter", "string_value")

            # Values should be different
            ns1_value = await client_ns1.get("counter")
            ns2_value = await client_ns2.get("counter")

            assert ns1_value == 100, "Namespace1 should have integer value"
            assert ns2_value == "string_value", "Namespace2 should have string value"

            # Operations should be isolated
            new_value = await client_ns1.incr("counter", 5)
            assert new_value == 105, "Increment should work on integer in namespace1"

            # Namespace2 value should be unchanged
            ns2_value_after = await client_ns2.get("counter")
            assert ns2_value_after == "string_value", "Namespace2 value should be unchanged"
        finally:
            # Cleanup
            await client_ns1.delete("counter")
            await client_ns2.delete("counter")

    @pytest.mark.asyncio
    async def test_no_namespace_client_isolated_from_namespaced_keys(self):
        """Test client without namespace doesn't see namespaced keys."""
        # Create clients
        client_no_ns = await KvClient.connect("127.0.0.1:11010")
        client_with_ns = await KvClient.connect("127.0.0.1:11010/isolated")

        try:
            # Set key in namespaced client
            await client_with_ns.set("test_key", "namespaced_value")

            # No-namespace client should not see it
            value_no_ns = await client_no_ns.get("test_key")
            assert value_no_ns is None, "No-namespace client should not see namespaced key"

            # Set key in no-namespace client
            await client_no_ns.set("test_key", "root_value")

            # Namespaced client should not see it
            value_with_ns = await client_with_ns.get("test_key")
            assert value_with_ns == "namespaced_value", "Namespaced client should only see its own value"

            # No-namespace client should see its own value
            value_no_ns_after = await client_no_ns.get("test_key")
            assert value_no_ns_after == "root_value", "No-namespace client should see its own value"
        finally:
            # Cleanup
            await client_no_ns.delete("test_key")
            await client_with_ns.delete("test_key")

    @pytest.mark.asyncio
    async def test_nested_namespaces_are_isolated(self):
        """Test nested namespaces are properly isolated."""
        # Create clients with nested namespaces
        client_prod = await KvClient.connect("127.0.0.1:11010/prod")
        client_prod_cache = await KvClient.connect("127.0.0.1:11010/prod/cache")
        client_prod_db = await KvClient.connect("127.0.0.1:11010/prod/db")

        try:
            # Set values in each namespace
            await client_prod.set("config", "prod_config")
            await client_prod_cache.set("config", "cache_config")
            await client_prod_db.set("config", "db_config")

            # Verify isolation
            prod_value = await client_prod.get("config")
            cache_value = await client_prod_cache.get("config")
            db_value = await client_prod_db.get("config")

            assert prod_value == "prod_config", "Prod namespace should have its value"
            assert cache_value == "cache_config", "Prod/cache namespace should have its value"
            assert db_value == "db_config", "Prod/db namespace should have its value"
        finally:
            # Cleanup
            await client_prod.delete("config")
            await client_prod_cache.delete("config")
            await client_prod_db.delete("config")

    @pytest.mark.asyncio
    async def test_exists_respects_namespace(self):
        """Test exists() operation respects namespace boundaries."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/ns1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/ns2")

        try:
            # Set key only in ns1
            await client_ns1.set("exists_key", "value")

            # Should exist in ns1
            exists_ns1 = await client_ns1.exists("exists_key")
            assert exists_ns1 is True, "Key should exist in ns1"

            # Should not exist in ns2
            exists_ns2 = await client_ns2.exists("exists_key")
            assert exists_ns2 is False, "Key should not exist in ns2"
        finally:
            # Cleanup
            await client_ns1.delete("exists_key")

    @pytest.mark.asyncio
    async def test_delete_respects_namespace(self):
        """Test delete() operation respects namespace boundaries."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/ns1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/ns2")

        try:
            # Set same key in both namespaces
            await client_ns1.set("delete_key", "value1")
            await client_ns2.set("delete_key", "value2")

            # Delete from ns1
            deleted = await client_ns1.delete("delete_key")
            assert deleted is True, "Delete should succeed in ns1"

            # Should not exist in ns1
            exists_ns1 = await client_ns1.exists("delete_key")
            assert exists_ns1 is False, "Key should not exist in ns1 after delete"

            # Should still exist in ns2
            exists_ns2 = await client_ns2.exists("delete_key")
            assert exists_ns2 is True, "Key should still exist in ns2"

            value_ns2 = await client_ns2.get("delete_key")
            assert value_ns2 == "value2", "Value in ns2 should be unchanged"
        finally:
            # Cleanup
            await client_ns2.delete("delete_key")


class TestLockIsolation:
    """Test lock isolation between namespaces."""

    @pytest.mark.asyncio
    async def test_locks_in_different_namespaces_dont_conflict(self):
        """Test locks in different namespaces are independent."""
        client_tasks = await KvClient.connect("127.0.0.1:11010/tasks")
        client_cache = await KvClient.connect("127.0.0.1:11010/cache")

        try:
            # Acquire lock on same key name in both namespaces
            lock_tasks = await client_tasks.lock("resource_lock", "owner_tasks", ttl=10.0)
            lock_cache = await client_cache.lock("resource_lock", "owner_cache", ttl=10.0)

            # Both should succeed (different namespaces)
            assert lock_tasks is True, "Lock should succeed in tasks namespace"
            assert lock_cache is True, "Lock should succeed in cache namespace"

            # Verify both locks exist
            exists_tasks = await client_tasks.exists("resource_lock")
            exists_cache = await client_cache.exists("resource_lock")

            assert exists_tasks is True, "Lock should exist in tasks namespace"
            assert exists_cache is True, "Lock should exist in cache namespace"
        finally:
            # Cleanup
            await client_tasks.unlock("resource_lock", "owner_tasks")
            await client_cache.unlock("resource_lock", "owner_cache")

    @pytest.mark.asyncio
    async def test_lock_in_one_namespace_doesnt_block_another(self):
        """Test lock in one namespace doesn't block same key in another namespace."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/namespace1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/namespace2")

        try:
            # Owner1 acquires lock in ns1
            result1 = await client_ns1.lock("lock:test", "owner1", ttl=10.0)
            assert result1 is True, "Lock should succeed in namespace1"

            # Owner2 should be able to acquire same lock in ns2
            result2 = await client_ns2.lock("lock:test", "owner2", ttl=10.0)
            assert result2 is True, "Lock should succeed in namespace2 (different namespace)"

            # Verify both locks are held
            exists_ns1 = await client_ns1.exists("lock:test")
            exists_ns2 = await client_ns2.exists("lock:test")

            assert exists_ns1 is True, "Lock should exist in namespace1"
            assert exists_ns2 is True, "Lock should exist in namespace2"
        finally:
            # Cleanup
            await client_ns1.unlock("lock:test", "owner1")
            await client_ns2.unlock("lock:test", "owner2")

    @pytest.mark.asyncio
    async def test_unlock_respects_namespace_boundary(self):
        """Test unlock only affects lock in same namespace."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/ns1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/ns2")

        try:
            # Acquire locks in both namespaces
            await client_ns1.lock("lock:shared", "owner1", ttl=10.0)
            await client_ns2.lock("lock:shared", "owner2", ttl=10.0)

            # Unlock in ns1
            unlocked = await client_ns1.unlock("lock:shared", "owner1")
            assert unlocked is True, "Unlock should succeed in ns1"

            # Lock should be released in ns1
            exists_ns1 = await client_ns1.exists("lock:shared")
            assert exists_ns1 is False, "Lock should be released in ns1"

            # Lock should still exist in ns2
            exists_ns2 = await client_ns2.exists("lock:shared")
            assert exists_ns2 is True, "Lock should still exist in ns2"
        finally:
            # Cleanup
            await client_ns2.unlock("lock:shared", "owner2")

    @pytest.mark.asyncio
    async def test_extend_lock_respects_namespace(self):
        """Test extend_lock only affects lock in same namespace."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/ns1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/ns2")

        try:
            # Acquire locks in both namespaces with short TTL
            await client_ns1.lock("lock:extend", "owner1", ttl=1.0)
            await client_ns2.lock("lock:extend", "owner2", ttl=1.0)

            # Extend lock in ns1
            extended = await client_ns1.extend_lock("lock:extend", "owner1", ttl=10.0)
            assert extended is True, "Extend should succeed in ns1"

            # Wait for original TTL to expire
            await asyncio.sleep(1.2)

            # Lock in ns1 should still exist (extended)
            exists_ns1 = await client_ns1.exists("lock:extend")
            assert exists_ns1 is True, "Lock should still exist in ns1 after extension"

            # Lock in ns2 should be expired (not extended)
            exists_ns2 = await client_ns2.exists("lock:extend")
            assert exists_ns2 is False, "Lock should expire in ns2 (not extended)"
        finally:
            # Cleanup
            try:
                await client_ns1.unlock("lock:extend", "owner1")
            except:
                pass

    @pytest.mark.asyncio
    async def test_lock_context_manager_with_namespace(self):
        """Test Lock context manager works correctly with namespaces."""
        client_tasks = await KvClient.connect("127.0.0.1:11010/tasks")
        client_cache = await KvClient.connect("127.0.0.1:11010/cache")

        try:
            # Use lock context manager in tasks namespace
            async with Lock(client_tasks, "lock:ctx", "owner_tasks", ttl=10.0) as acquired_tasks:
                assert acquired_tasks is True, "Lock should be acquired in tasks namespace"

                # Lock in cache namespace with same key should also work
                async with Lock(client_cache, "lock:ctx", "owner_cache", ttl=10.0) as acquired_cache:
                    assert acquired_cache is True, "Lock should be acquired in cache namespace"

                    # Both locks should exist
                    exists_tasks = await client_tasks.exists("lock:ctx")
                    exists_cache = await client_cache.exists("lock:ctx")

                    assert exists_tasks is True, "Lock should exist in tasks namespace"
                    assert exists_cache is True, "Lock should exist in cache namespace"

                # Cache lock should be released
                exists_cache_after = await client_cache.exists("lock:ctx")
                assert exists_cache_after is False, "Cache lock should be released"

                # Tasks lock should still exist
                exists_tasks_after = await client_tasks.exists("lock:ctx")
                assert exists_tasks_after is True, "Tasks lock should still exist"

            # Both locks should be released
            exists_tasks_final = await client_tasks.exists("lock:ctx")
            exists_cache_final = await client_cache.exists("lock:ctx")

            assert exists_tasks_final is False, "Tasks lock should be released"
            assert exists_cache_final is False, "Cache lock should be released"
        finally:
            # Cleanup (in case of test failure)
            try:
                await client_tasks.delete("lock:ctx")
                await client_cache.delete("lock:ctx")
            except:
                pass


class TestNamespaceConcurrency:
    """Test namespace behavior under concurrent access."""

    @pytest.mark.asyncio
    async def test_concurrent_operations_across_namespaces(self):
        """Test concurrent operations across different namespaces work independently."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/concurrent1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/concurrent2")

        try:
            # Initialize counters in both namespaces
            await client_ns1.set("counter", 0)
            await client_ns2.set("counter", 0)

            async def increment_namespace(client, namespace_name, iterations):
                for _ in range(iterations):
                    await client.incr("counter", 1)

            # Run concurrent increments in both namespaces
            tasks = [
                increment_namespace(client_ns1, "ns1", 50),
                increment_namespace(client_ns2, "ns2", 50),
            ]
            await asyncio.gather(*tasks)

            # Each namespace should have its own correct count
            count_ns1 = await client_ns1.get("counter")
            count_ns2 = await client_ns2.get("counter")

            assert count_ns1 == 50, "Namespace1 counter should be 50"
            assert count_ns2 == 50, "Namespace2 counter should be 50"
        finally:
            # Cleanup
            await client_ns1.delete("counter")
            await client_ns2.delete("counter")

    @pytest.mark.asyncio
    async def test_concurrent_lock_attempts_across_namespaces(self):
        """Test concurrent lock attempts across namespaces succeed independently."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/locks1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/locks2")

        results = []

        async def try_acquire_lock(client, namespace, owner):
            acquired = await client.lock("shared_lock", owner, ttl=10.0)
            results.append((namespace, owner, acquired))
            if acquired:
                await asyncio.sleep(0.1)
                await client.unlock("shared_lock", owner)

        try:
            # Try to acquire lock concurrently in both namespaces
            tasks = [
                try_acquire_lock(client_ns1, "ns1", "owner1"),
                try_acquire_lock(client_ns2, "ns2", "owner2"),
            ]
            await asyncio.gather(*tasks)

            # Both should succeed (different namespaces)
            ns1_result = next((r for r in results if r[0] == "ns1"), None)
            ns2_result = next((r for r in results if r[0] == "ns2"), None)

            assert ns1_result is not None, "Namespace1 result should exist"
            assert ns2_result is not None, "Namespace2 result should exist"
            assert ns1_result[2] is True, "Lock should succeed in namespace1"
            assert ns2_result[2] is True, "Lock should succeed in namespace2"
        finally:
            # Cleanup
            try:
                await client_ns1.delete("shared_lock")
                await client_ns2.delete("shared_lock")
            except:
                pass


class TestNamespaceEdgeCases:
    """Test edge cases and special characters in namespaces."""

    @pytest.mark.asyncio
    async def test_namespace_with_special_characters(self):
        """Test namespace with special characters works correctly."""
        # Create client with special characters in namespace
        client = await KvClient.connect("127.0.0.1:11010/app-v1.0_test")

        try:
            # Set and get value
            await client.set("test_key", "test_value")
            value = await client.get("test_key")

            assert value == "test_value", "Value should be set/retrieved correctly"
            assert client.namespace == "app-v1.0_test", "Namespace should preserve special characters"
        finally:
            # Cleanup
            await client.delete("test_key")

    @pytest.mark.asyncio
    async def test_deeply_nested_namespace(self):
        """Test deeply nested namespace works correctly."""
        # Create client with deeply nested namespace
        client = await KvClient.connect("127.0.0.1:11010/env/prod/region/us-west/service/api")

        try:
            # Set and get value
            await client.set("config", "api_config")
            value = await client.get("config")

            assert value == "api_config", "Value should be set/retrieved correctly"
            assert client.namespace == "env/prod/region/us-west/service/api", "Namespace should be correct"
        finally:
            # Cleanup
            await client.delete("config")

    @pytest.mark.asyncio
    async def test_setnx_respects_namespace(self):
        """Test setnx operation respects namespace boundaries."""
        client_ns1 = await KvClient.connect("127.0.0.1:11010/setnx1")
        client_ns2 = await KvClient.connect("127.0.0.1:11010/setnx2")

        try:
            # setnx in ns1
            result1 = await client_ns1.setnx("setnx_key", "value1")
            assert result1 is True, "setnx should succeed in ns1"

            # setnx with same key in ns2 should also succeed (different namespace)
            result2 = await client_ns2.setnx("setnx_key", "value2")
            assert result2 is True, "setnx should succeed in ns2 (different namespace)"

            # Verify values are isolated
            value1 = await client_ns1.get("setnx_key")
            value2 = await client_ns2.get("setnx_key")

            assert value1 == "value1", "Namespace1 should have its value"
            assert value2 == "value2", "Namespace2 should have its value"
        finally:
            # Cleanup
            await client_ns1.delete("setnx_key")
            await client_ns2.delete("setnx_key")
