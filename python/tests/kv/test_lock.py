"""
Integration tests for KV Store Lock API.

Tests distributed locking functionality including:
- SETNX (Set if not exists)
- Lock acquisition and release
- Lock extension
- Lock context manager
- Reentrant locks
"""

import pytest
from cclab.probe import expect
import asyncio
from cclab.ion import KvClient, Lock


class TestSetnx:
    """Test SETNX (Set if not exists) functionality."""

    @pytest.mark.asyncio
    async def test_setnx_on_new_key(self, kv_client: KvClient):
        """Test setnx on a new key returns True."""
        # SETNX on new key should succeed
        result = await kv_client.setnx("test_setnx", "value1")
        assert result is True, "setnx should return True for new key"

        # Verify value was set
        value = await kv_client.get("test_setnx")
        assert value == "value1", "Value should be set correctly"

    @pytest.mark.asyncio
    async def test_setnx_on_existing_key(self, kv_client: KvClient):
        """Test setnx on an existing key returns False."""
        # Set initial value
        await kv_client.set("test_setnx_existing", "value1")

        # SETNX on existing key should fail
        result = await kv_client.setnx("test_setnx_existing", "value2")
        assert result is False, "setnx should return False for existing key"

        # Verify original value unchanged
        value = await kv_client.get("test_setnx_existing")
        assert value == "value1", "Original value should remain unchanged"

    @pytest.mark.asyncio
    async def test_setnx_with_ttl(self, kv_client: KvClient):
        """Test setnx with TTL expires correctly."""
        # Set key with short TTL
        result = await kv_client.setnx("test_setnx_ttl", "value1", ttl=0.5)
        assert result is True, "setnx with TTL should succeed"

        # Key should exist immediately
        exists = await kv_client.exists("test_setnx_ttl")
        assert exists is True, "Key should exist immediately"

        # Wait for expiration
        await asyncio.sleep(0.6)

        # Key should be gone
        exists = await kv_client.exists("test_setnx_ttl")
        assert exists is False, "Key should expire after TTL"

        # SETNX should succeed again
        result = await kv_client.setnx("test_setnx_ttl", "value2")
        assert result is True, "setnx should succeed after expiration"


class TestLockAcquisition:
    """Test lock acquisition functionality."""

    @pytest.mark.asyncio
    async def test_lock_on_new_key(self, kv_client: KvClient):
        """Test lock() on new key returns True."""
        # Acquire lock on new key
        result = await kv_client.lock("test_lock", "owner1", ttl=10.0)
        assert result is True, "lock should succeed on new key"

        # Verify lock is held (key exists)
        exists = await kv_client.exists("test_lock")
        assert exists is True, "Lock key should exist"

    @pytest.mark.asyncio
    async def test_lock_reentrant_same_owner(self, kv_client: KvClient):
        """Test lock() on already locked key by same owner returns False (not reentrant)."""
        # Acquire lock
        result1 = await kv_client.lock("test_lock_reentrant", "owner1", ttl=10.0)
        assert result1 is True, "First lock should succeed"

        # Acquire same lock again with same owner (not reentrant in current implementation)
        result2 = await kv_client.lock("test_lock_reentrant", "owner1", ttl=10.0)
        assert result2 is False, "Lock is not reentrant, even for same owner"

    @pytest.mark.asyncio
    async def test_lock_different_owner(self, kv_client: KvClient):
        """Test lock() on locked key by different owner returns False."""
        # Owner 1 acquires lock
        result1 = await kv_client.lock("test_lock_different", "owner1", ttl=10.0)
        assert result1 is True, "First lock should succeed"

        # Owner 2 tries to acquire same lock
        result2 = await kv_client.lock("test_lock_different", "owner2", ttl=10.0)
        assert result2 is False, "Lock should fail for different owner"

        # Verify lock still held by owner1
        exists = await kv_client.exists("test_lock_different")
        assert exists is True, "Lock should still be held"


class TestLockRelease:
    """Test lock release functionality."""

    @pytest.mark.asyncio
    async def test_unlock_by_owner(self, kv_client: KvClient):
        """Test unlock() by owner returns True."""
        # Acquire lock
        await kv_client.lock("lock:resource", "owner1", ttl=10.0)

        # Release lock by owner
        result = await kv_client.unlock("lock:resource", "owner1")
        assert result is True, "unlock should succeed for owner"

        # Verify lock is released
        exists = await kv_client.exists("lock:resource")
        assert exists is False, "Lock key should be deleted after unlock"

    @pytest.mark.asyncio
    async def test_unlock_by_different_owner(self, kv_client: KvClient):
        """Test unlock() by different owner raises error."""
        # Acquire lock with owner1
        await kv_client.lock("lock:resource:1", "owner1", ttl=10.0)

        # Try to release with different owner (should raise error)
        exception_raised = False
        try:
            await kv_client.unlock("lock:resource:1", "owner2")
        except RuntimeError:
            exception_raised = True
        assert exception_raised, "unlock with different owner should raise RuntimeError"

        # Verify lock still held
        exists = await kv_client.exists("lock:resource:1")
        assert exists is True, "Lock should still be held"

    @pytest.mark.asyncio
    async def test_unlock_nonexistent_lock(self, kv_client: KvClient):
        """Test unlock() on non-existent lock returns False."""
        # Try to unlock non-existent lock
        result = await kv_client.unlock("lock:nonexistent", "owner1")
        assert result is False, "unlock should return False for non-existent lock"


class TestLockExtend:
    """Test lock extension functionality."""

    @pytest.mark.asyncio
    async def test_extend_lock_by_owner(self, kv_client: KvClient):
        """Test extend_lock() by owner returns True."""
        # Acquire lock with short TTL
        await kv_client.lock("lock:extend", "owner1", ttl=1.0)

        # Extend lock
        result = await kv_client.extend_lock("lock:extend", "owner1", ttl=10.0)
        assert result is True, "extend_lock should succeed for owner"

    @pytest.mark.asyncio
    async def test_extend_lock_by_different_owner(self, kv_client: KvClient):
        """Test extend_lock() by different owner raises error."""
        # Acquire lock with owner1
        await kv_client.lock("lock:resource:2", "owner1", ttl=10.0)

        # Try to extend with different owner (should raise error)
        exception_raised = False
        try:
            await kv_client.extend_lock("lock:resource:2", "owner2", ttl=10.0)
        except RuntimeError:
            exception_raised = True
        assert exception_raised, "extend_lock with different owner should raise RuntimeError"

    @pytest.mark.asyncio
    async def test_extend_lock_actually_extends_ttl(self, kv_client: KvClient):
        """Test extend_lock() actually extends the TTL."""
        # Acquire lock with short TTL
        await kv_client.lock("lock:task", "owner1", ttl=0.5)

        # Wait a bit
        await asyncio.sleep(0.3)

        # Extend lock
        result = await kv_client.extend_lock("lock:task", "owner1", ttl=2.0)
        assert result is True, "extend_lock should succeed"

        # Wait past original TTL
        await asyncio.sleep(0.5)

        # Lock should still exist (extended)
        exists = await kv_client.exists("lock:task")
        assert exists is True, "Lock should still exist after extension"

        # Wait for extended TTL to expire
        await asyncio.sleep(1.6)

        # Lock should now be gone
        exists = await kv_client.exists("lock:task")
        assert exists is False, "Lock should expire after extended TTL"


class TestLockContextManager:
    """Test Python Lock context manager."""

    @pytest.mark.asyncio
    async def test_context_manager_acquires_and_releases(self, kv_client: KvClient):
        """Test context manager acquires and releases lock."""
        # Use lock context manager
        async with Lock(kv_client, "lock:resource", "owner1", ttl=10.0) as acquired:
            assert acquired is True, "Lock should be acquired"

            # Verify lock is held
            exists = await kv_client.exists("lock:resource")
            assert exists is True, "Lock should exist inside context"

        # After context, lock should be released
        exists = await kv_client.exists("lock:resource")
        assert exists is False, "Lock should be released after context"

    @pytest.mark.asyncio
    async def test_context_manager_extend(self, kv_client: KvClient):
        """Test Lock.extend() works within context."""
        # Use lock context manager with short TTL
        lock = Lock(kv_client, "lock:resource", "owner1", ttl=0.5)

        async with lock as acquired:
            assert acquired is True, "Lock should be acquired"

            # Wait a bit
            await asyncio.sleep(0.3)

            # Extend lock
            result = await lock.extend(ttl=10.0)
            assert result is True, "extend should succeed"

            # Wait past original TTL
            await asyncio.sleep(0.5)

            # Lock should still exist
            exists = await kv_client.exists("lock:resource")
            assert exists is True, "Lock should still exist after extension"

        # After context, lock should be released
        exists = await kv_client.exists("lock:resource")
        assert exists is False, "Lock should be released after context"

    @pytest.mark.asyncio
    async def test_context_manager_failed_acquisition(self, kv_client: KvClient):
        """Test context manager when lock acquisition fails."""
        # Owner 1 acquires lock
        await kv_client.lock("lock:resource", "owner1", ttl=10.0)

        # Owner 2 tries to acquire with context manager
        async with Lock(kv_client, "lock:resource", "owner2", ttl=10.0) as acquired:
            assert acquired is False, "Lock acquisition should fail"

            # Should not have lock
            # We can't directly check ownership, but we know owner1 still holds it

        # Lock should still exist (held by owner1)
        exists = await kv_client.exists("lock:resource")
        assert exists is True, "Lock should still be held by owner1"

        # Owner 1 releases
        await kv_client.unlock("lock:resource", "owner1")

    @pytest.mark.asyncio
    async def test_nested_context_managers(self, kv_client: KvClient):
        """Test nested context managers work correctly."""
        # Acquire first lock
        async with Lock(kv_client, "lock:nested:1", "owner1", ttl=10.0) as acquired1:
            assert acquired1 is True, "First lock should be acquired"

            # Acquire second lock inside first
            async with Lock(kv_client, "lock:nested:2", "owner1", ttl=10.0) as acquired2:
                assert acquired2 is True, "Second lock should be acquired"

                # Both locks should exist
                exists1 = await kv_client.exists("lock:nested:1")
                exists2 = await kv_client.exists("lock:nested:2")
                assert exists1 is True, "First lock should exist"
                assert exists2 is True, "Second lock should exist"

            # After inner context, only first lock should exist
            exists1 = await kv_client.exists("lock:nested:1")
            exists2 = await kv_client.exists("lock:nested:2")
            assert exists1 is True, "First lock should still exist"
            assert exists2 is False, "Second lock should be released"

        # After outer context, no locks should exist
        exists1 = await kv_client.exists("lock:nested:1")
        exists2 = await kv_client.exists("lock:nested:2")
        assert exists1 is False, "First lock should be released"
        assert exists2 is False, "Second lock should be released"

    @pytest.mark.asyncio
    async def test_context_manager_with_exception(self, kv_client: KvClient):
        """Test context manager releases lock even with exception."""
        try:
            async with Lock(kv_client, "lock:resource", "owner1", ttl=10.0) as acquired:
                assert acquired is True, "Lock should be acquired"

                # Raise exception inside context
                raise ValueError("Test exception")
        except ValueError:
            pass  # Expected exception

        # Lock should be released despite exception
        exists = await kv_client.exists("lock:resource")
        assert exists is False, "Lock should be released after exception"


class TestLockConcurrency:
    """Test lock behavior under concurrent access."""

    @pytest.mark.asyncio
    async def test_concurrent_lock_attempts(self, kv_client: KvClient):
        """Test multiple concurrent lock attempts."""
        acquired_count = 0

        async def try_acquire_lock(owner: str):
            nonlocal acquired_count
            result = await kv_client.lock("lock:resource", owner, ttl=10.0)
            if result:
                acquired_count += 1
                # Hold lock briefly
                await asyncio.sleep(0.1)
                await kv_client.unlock("lock:resource", owner)

        # Try to acquire lock concurrently from 5 workers
        tasks = [try_acquire_lock(f"owner{i}") for i in range(5)]
        await asyncio.gather(*tasks)

        # Only one should have succeeded
        assert acquired_count == 1, "Only one worker should acquire the lock"

    @pytest.mark.asyncio
    async def test_lock_queue_behavior(self, kv_client: KvClient):
        """Test lock queue behavior with sequential attempts."""
        results = []

        async def acquire_and_hold(owner: str, hold_time: float):
            acquired = await kv_client.lock("lock:resource", owner, ttl=10.0)
            results.append((owner, acquired))
            if acquired:
                await asyncio.sleep(hold_time)
                await kv_client.unlock("lock:resource", owner)

        # Owner 1 acquires lock
        task1 = asyncio.create_task(acquire_and_hold("owner1", 0.3))

        # Wait a bit for owner1 to acquire
        await asyncio.sleep(0.05)

        # Owner 2 tries (should fail while owner1 holds)
        task2 = asyncio.create_task(acquire_and_hold("owner2", 0.0))

        # Wait for owner1's lock to be released
        await task1

        # Owner 2 should have failed
        await task2

        # Owner 3 tries after owner1 released (should succeed)
        task3 = asyncio.create_task(acquire_and_hold("owner3", 0.0))
        await task3

        # Check results
        assert results[0] == ("owner1", True), "Owner1 should acquire lock"
        assert results[1] == ("owner2", False), "Owner2 should fail while owner1 holds"
        assert results[2] == ("owner3", True), "Owner3 should acquire after owner1 releases"
