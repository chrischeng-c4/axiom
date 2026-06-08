"""
Integration tests for batch operations (MGET, MSET, MDEL).

These tests verify:
1. Correctness of batch operations
2. Edge cases (empty batches, missing keys, etc.)
3. Performance benefits vs individual operations
4. Namespace support
5. TTL handling in MSET

Run with: pytest tests/kv/test_batch_operations.py -v -s
"""

import pytest
import asyncio
import time
from typing import List
from cclab.ion import KvClient


# ============================================================================
# Correctness Tests
# ============================================================================


class TestBatchCorrectness:
    """Test correctness of batch operations."""

    @pytest.mark.asyncio
    async def test_mget_basic(self, kv_client: KvClient):
        """Test MGET retrieves multiple values correctly."""
        # Setup: Set 3 keys
        await kv_client.set("batch:key1", "value1")
        await kv_client.set("batch:key2", "value2")
        await kv_client.set("batch:key3", "value3")

        # Test: MGET all 3 keys
        results = await kv_client.mget(["batch:key1", "batch:key2", "batch:key3"])

        # Verify
        assert len(results) == 3
        assert results[0] == "value1"
        assert results[1] == "value2"
        assert results[2] == "value3"

        # Cleanup
        await kv_client.delete("batch:key1")
        await kv_client.delete("batch:key2")
        await kv_client.delete("batch:key3")

    @pytest.mark.asyncio
    async def test_mget_with_missing_keys(self, kv_client: KvClient):
        """Test MGET returns None for missing keys."""
        # Setup: Only set key2
        await kv_client.set("batch:key2", "value2")

        # Test: MGET with missing keys
        results = await kv_client.mget(["batch:key1", "batch:key2", "batch:key3"])

        # Verify: None for missing keys
        assert len(results) == 3
        assert results[0] is None
        assert results[1] == "value2"
        assert results[2] is None

        # Cleanup
        await kv_client.delete("batch:key2")

    @pytest.mark.asyncio
    async def test_mset_basic(self, kv_client: KvClient):
        """Test MSET stores multiple key-value pairs."""
        # Test: MSET 3 pairs
        pairs = [
            ("batch:k1", "v1"),
            ("batch:k2", "v2"),
            ("batch:k3", "v3"),
        ]
        await kv_client.mset(pairs)

        # Verify: All keys exist with correct values
        assert await kv_client.get("batch:k1") == "v1"
        assert await kv_client.get("batch:k2") == "v2"
        assert await kv_client.get("batch:k3") == "v3"

        # Cleanup
        await kv_client.delete("batch:k1")
        await kv_client.delete("batch:k2")
        await kv_client.delete("batch:k3")

    @pytest.mark.asyncio
    async def test_mset_with_ttl(self, kv_client: KvClient):
        """Test MSET respects TTL parameter."""
        # Test: MSET with 1 second TTL
        pairs = [("batch:ttl1", "v1"), ("batch:ttl2", "v2")]
        await kv_client.mset(pairs, ttl=1.0)

        # Verify: Keys exist immediately
        assert await kv_client.get("batch:ttl1") == "v1"
        assert await kv_client.get("batch:ttl2") == "v2"

        # Wait for expiration
        await asyncio.sleep(1.1)

        # Verify: Keys expired
        assert await kv_client.get("batch:ttl1") is None
        assert await kv_client.get("batch:ttl2") is None

    @pytest.mark.asyncio
    async def test_mdel_basic(self, kv_client: KvClient):
        """Test MDEL deletes multiple keys and returns count."""
        # Setup: Set 3 keys
        await kv_client.set("batch:del1", "v1")
        await kv_client.set("batch:del2", "v2")
        await kv_client.set("batch:del3", "v3")

        # Test: MDEL all 3 keys
        deleted = await kv_client.mdel(["batch:del1", "batch:del2", "batch:del3"])

        # Verify: All 3 deleted
        assert deleted == 3
        assert await kv_client.get("batch:del1") is None
        assert await kv_client.get("batch:del2") is None
        assert await kv_client.get("batch:del3") is None

    @pytest.mark.asyncio
    async def test_mdel_with_missing_keys(self, kv_client: KvClient):
        """Test MDEL only counts actually deleted keys."""
        # Setup: Only set del2
        await kv_client.set("batch:del2", "v2")

        # Test: MDEL including missing keys
        deleted = await kv_client.mdel(["batch:del1", "batch:del2", "batch:del3"])

        # Verify: Only 1 deleted
        assert deleted == 1


# ============================================================================
# Edge Cases Tests
# ============================================================================


class TestBatchEdgeCases:
    """Test edge cases and error handling."""

    @pytest.mark.asyncio
    async def test_mget_empty_list(self, kv_client: KvClient):
        """Test MGET with empty key list."""
        results = await kv_client.mget([])
        assert results == []

    @pytest.mark.asyncio
    async def test_mset_empty_list(self, kv_client: KvClient):
        """Test MSET with empty pairs list."""
        await kv_client.mset([])
        # Should not raise error

    @pytest.mark.asyncio
    async def test_mdel_empty_list(self, kv_client: KvClient):
        """Test MDEL with empty key list."""
        deleted = await kv_client.mdel([])
        assert deleted == 0

    @pytest.mark.asyncio
    async def test_mget_single_key(self, kv_client: KvClient):
        """Test MGET with single key (edge of batch)."""
        await kv_client.set("batch:single", "value")
        results = await kv_client.mget(["batch:single"])
        assert len(results) == 1
        assert results[0] == "value"
        await kv_client.delete("batch:single")

    @pytest.mark.asyncio
    async def test_mget_large_batch(self, kv_client: KvClient):
        """Test MGET with large batch (1000 keys)."""
        # Setup: Create 1000 keys
        batch_size = 1000
        keys = [f"batch:large:{i}" for i in range(batch_size)]
        pairs = [(key, f"value{i}") for i, key in enumerate(keys)]
        await kv_client.mset(pairs)

        # Test: MGET 1000 keys
        results = await kv_client.mget(keys)

        # Verify: All 1000 values correct
        assert len(results) == batch_size
        for i, result in enumerate(results):
            assert result == f"value{i}", f"Mismatch at index {i}"

        # Cleanup
        deleted = await kv_client.mdel(keys)
        assert deleted == batch_size

    @pytest.mark.asyncio
    async def test_mset_overwrites_existing(self, kv_client: KvClient):
        """Test MSET overwrites existing keys."""
        # Setup: Set initial values
        await kv_client.set("batch:ow1", "old1")
        await kv_client.set("batch:ow2", "old2")

        # Test: MSET with new values
        await kv_client.mset([("batch:ow1", "new1"), ("batch:ow2", "new2")])

        # Verify: Values overwritten
        assert await kv_client.get("batch:ow1") == "new1"
        assert await kv_client.get("batch:ow2") == "new2"

        # Cleanup
        await kv_client.delete("batch:ow1")
        await kv_client.delete("batch:ow2")


# ============================================================================
# Data Type Tests
# ============================================================================


class TestBatchDataTypes:
    """Test batch operations with different value types."""

    @pytest.mark.asyncio
    async def test_mget_mset_integers(self, kv_client: KvClient):
        """Test MGET/MSET with integer values."""
        pairs = [("batch:int1", 123), ("batch:int2", 456)]
        await kv_client.mset(pairs)

        results = await kv_client.mget(["batch:int1", "batch:int2"])
        assert results == [123, 456]

        await kv_client.mdel(["batch:int1", "batch:int2"])

    @pytest.mark.asyncio
    async def test_mget_mset_floats(self, kv_client: KvClient):
        """Test MGET/MSET with float values."""
        pairs = [("batch:float1", 3.14), ("batch:float2", 2.718)]
        await kv_client.mset(pairs)

        results = await kv_client.mget(["batch:float1", "batch:float2"])
        assert results == [3.14, 2.718]

        await kv_client.mdel(["batch:float1", "batch:float2"])

    @pytest.mark.asyncio
    async def test_mget_mset_booleans(self, kv_client: KvClient):
        """Test MGET/MSET with boolean values."""
        pairs = [("batch:bool1", True), ("batch:bool2", False)]
        await kv_client.mset(pairs)

        results = await kv_client.mget(["batch:bool1", "batch:bool2"])
        assert results == [True, False]

        await kv_client.mdel(["batch:bool1", "batch:bool2"])

    @pytest.mark.asyncio
    async def test_mget_mset_mixed_types(self, kv_client: KvClient):
        """Test MGET/MSET with mixed value types."""
        pairs = [
            ("batch:mix1", "string"),
            ("batch:mix2", 42),
            ("batch:mix3", 3.14),
            ("batch:mix4", True),
        ]
        await kv_client.mset(pairs)

        results = await kv_client.mget(
            ["batch:mix1", "batch:mix2", "batch:mix3", "batch:mix4"]
        )
        assert results == ["string", 42, 3.14, True]

        await kv_client.mdel(["batch:mix1", "batch:mix2", "batch:mix3", "batch:mix4"])


# ============================================================================
# Performance Tests
# ============================================================================


@pytest.mark.benchmark
class TestBatchPerformance:
    """Test performance benefits of batch operations."""

    @pytest.mark.asyncio
    async def test_mget_vs_individual_get(self, kv_client: KvClient):
        """
        Compare MGET vs N individual GETs.

        Expected: MGET should be significantly faster (10-100x)
        due to reduced network round-trips.
        """
        batch_size = 100
        keys = [f"batch:perf:get:{i}" for i in range(batch_size)]

        # Setup: Populate keys
        pairs = [(key, f"value{i}") for i, key in enumerate(keys)]
        await kv_client.mset(pairs)

        # Benchmark: Individual GETs
        start = time.perf_counter()
        for key in keys:
            await kv_client.get(key)
        individual_time = time.perf_counter() - start

        # Benchmark: MGET
        start = time.perf_counter()
        results = await kv_client.mget(keys)
        mget_time = time.perf_counter() - start

        # Calculate speedup
        speedup = individual_time / mget_time
        individual_ops = batch_size / individual_time
        mget_ops = batch_size / mget_time

        print(f"\n{'='*60}")
        print(f"MGET vs Individual GET ({batch_size} keys)")
        print(f"  Individual: {individual_ops:>10,.0f} ops/sec ({individual_time:.6f}s)")
        print(f"  MGET:       {mget_ops:>10,.0f} ops/sec ({mget_time:.6f}s)")
        print(f"  Speedup:    {speedup:>10.2f}x")
        print(f"{'='*60}")

        # Verify correctness
        assert len(results) == batch_size
        assert all(r == f"value{i}" for i, r in enumerate(results))

        # Assert performance benefit (at least 4x faster)
        # Note: Over network with real latency, speedup would be 10-100x
        assert speedup >= 4.0, f"MGET not fast enough: {speedup:.2f}x"

        # Cleanup
        await kv_client.mdel(keys)

    @pytest.mark.asyncio
    async def test_mset_vs_individual_set(self, kv_client: KvClient):
        """
        Compare MSET vs N individual SETs.

        Expected: MSET should be significantly faster (10-100x)
        due to reduced network round-trips.
        """
        batch_size = 100
        keys = [f"batch:perf:set:{i}" for i in range(batch_size)]
        pairs = [(key, f"value{i}") for i, key in enumerate(keys)]

        # Benchmark: Individual SETs
        start = time.perf_counter()
        for key, value in pairs:
            await kv_client.set(key, value)
        individual_time = time.perf_counter() - start

        # Cleanup before MSET test
        await kv_client.mdel(keys)

        # Benchmark: MSET
        start = time.perf_counter()
        await kv_client.mset(pairs)
        mset_time = time.perf_counter() - start

        # Calculate speedup
        speedup = individual_time / mset_time
        individual_ops = batch_size / individual_time
        mset_ops = batch_size / mset_time

        print(f"\n{'='*60}")
        print(f"MSET vs Individual SET ({batch_size} keys)")
        print(f"  Individual: {individual_ops:>10,.0f} ops/sec ({individual_time:.6f}s)")
        print(f"  MSET:       {mset_ops:>10,.0f} ops/sec ({mset_time:.6f}s)")
        print(f"  Speedup:    {speedup:>10.2f}x")
        print(f"{'='*60}")

        # Assert performance benefit (at least 4x faster)
        # Note: Over network with real latency, speedup would be 10-100x
        assert speedup >= 4.0, f"MSET not fast enough: {speedup:.2f}x"

        # Cleanup
        await kv_client.mdel(keys)

    @pytest.mark.asyncio
    async def test_mdel_vs_individual_delete(self, kv_client: KvClient):
        """
        Compare MDEL vs N individual DELETEs.

        Expected: MDEL should be significantly faster (10-100x)
        due to reduced network round-trips.
        """
        batch_size = 100
        keys = [f"batch:perf:del:{i}" for i in range(batch_size)]
        pairs = [(key, f"value{i}") for i, key in enumerate(keys)]

        # Setup: Populate keys for individual delete test
        await kv_client.mset(pairs)

        # Benchmark: Individual DELETEs
        start = time.perf_counter()
        for key in keys:
            await kv_client.delete(key)
        individual_time = time.perf_counter() - start

        # Setup: Populate keys for MDEL test
        await kv_client.mset(pairs)

        # Benchmark: MDEL
        start = time.perf_counter()
        deleted = await kv_client.mdel(keys)
        mdel_time = time.perf_counter() - start

        # Calculate speedup
        speedup = individual_time / mdel_time
        individual_ops = batch_size / individual_time
        mdel_ops = batch_size / mdel_time

        print(f"\n{'='*60}")
        print(f"MDEL vs Individual DELETE ({batch_size} keys)")
        print(f"  Individual: {individual_ops:>10,.0f} ops/sec ({individual_time:.6f}s)")
        print(f"  MDEL:       {mdel_ops:>10,.0f} ops/sec ({mdel_time:.6f}s)")
        print(f"  Speedup:    {speedup:>10.2f}x")
        print(f"{'='*60}")

        # Verify correctness
        assert deleted == batch_size

        # Assert performance benefit (at least 4x faster)
        # Note: Over network with real latency, speedup would be 10-100x
        assert speedup >= 4.0, f"MDEL not fast enough: {speedup:.2f}x"

    @pytest.mark.asyncio
    async def test_batch_scalability(self, kv_client: KvClient):
        """
        Test how batch operations scale with different sizes.

        Measures throughput for batch sizes: 10, 50, 100, 500, 1000
        """
        batch_sizes = [10, 50, 100, 500, 1000]

        print(f"\n{'='*60}")
        print("Batch Operation Scalability")
        print(f"{'='*60}")
        print(f"{'Batch Size':<12} {'MGET ops/sec':<15} {'MSET ops/sec':<15} {'MDEL ops/sec':<15}")
        print(f"{'-'*60}")

        for size in batch_sizes:
            keys = [f"batch:scale:{i}" for i in range(size)]
            pairs = [(key, f"v{i}") for i, key in enumerate(keys)]

            # MSET benchmark
            start = time.perf_counter()
            await kv_client.mset(pairs)
            mset_time = time.perf_counter() - start
            mset_ops = size / mset_time

            # MGET benchmark
            start = time.perf_counter()
            await kv_client.mget(keys)
            mget_time = time.perf_counter() - start
            mget_ops = size / mget_time

            # MDEL benchmark
            start = time.perf_counter()
            await kv_client.mdel(keys)
            mdel_time = time.perf_counter() - start
            mdel_ops = size / mdel_time

            print(f"{size:<12} {mget_ops:<15,.0f} {mset_ops:<15,.0f} {mdel_ops:<15,.0f}")

        print(f"{'='*60}\n")


# ============================================================================
# Namespace Tests
# ============================================================================


class TestBatchWithNamespace:
    """Test batch operations work correctly with namespaces."""

    @pytest.mark.asyncio
    async def test_mget_with_namespace(self):
        """Test MGET with namespace prefix."""
        # Create client with namespace
        client = await KvClient.connect("127.0.0.1:11010/testns")

        # Set keys (should be auto-prefixed)
        await client.set("key1", "value1")
        await client.set("key2", "value2")

        # Test: MGET should work with namespace
        results = await client.mget(["key1", "key2", "key3"])

        assert len(results) == 3
        assert results[0] == "value1"
        assert results[1] == "value2"
        assert results[2] is None

        # Cleanup
        await client.mdel(["key1", "key2"])

    @pytest.mark.asyncio
    async def test_mset_with_namespace(self):
        """Test MSET with namespace prefix."""
        client = await KvClient.connect("127.0.0.1:11010/testns2")

        # Test: MSET with namespace
        pairs = [("k1", "v1"), ("k2", "v2")]
        await client.mset(pairs)

        # Verify
        assert await client.get("k1") == "v1"
        assert await client.get("k2") == "v2"

        # Cleanup
        await client.mdel(["k1", "k2"])
