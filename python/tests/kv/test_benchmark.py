"""
Performance benchmark tests for KV store.

These tests measure and compare performance of:
1. Single client operations (SET, GET, mixed workload)
2. Connection pool operations (concurrent access, pool overhead)
3. data-bridge KV vs Redis (side-by-side comparison)

Run with: pytest tests/kv/test_benchmark.py -v -s -m benchmark
Skip with: pytest tests/kv/ -m "not benchmark"
"""

import pytest
import asyncio
import time
from typing import List, Tuple
from cclab.ion import KvClient, KvPool, PoolConfig

# Import redis-py for comparison
try:
    import redis.asyncio as redis

    REDIS_AVAILABLE = True
except ImportError:
    REDIS_AVAILABLE = False


# ============================================================================
# Helper Functions
# ============================================================================


def format_throughput(ops: int, elapsed: float) -> str:
    """Format throughput in a human-readable way."""
    ops_per_sec = ops / elapsed
    return f"{ops_per_sec:,.0f} ops/sec ({elapsed:.3f}s for {ops:,} ops)"


def print_comparison(
    name: str, db_ops: float, redis_ops: float, ops: int, db_time: float, redis_time: float
):
    """Print a formatted comparison table."""
    ratio = db_ops / redis_ops if redis_ops > 0 else 0
    print(f"\n{'='*60}")
    print(f"{name} Comparison ({ops:,} ops):")
    print(f"  data-bridge:  {db_ops:>10,.0f} ops/sec ({db_time:.3f}s)")
    print(f"  Redis:        {redis_ops:>10,.0f} ops/sec ({redis_time:.3f}s)")
    print(f"  Ratio:        {ratio:>10.2f}x")
    print(f"{'='*60}")


# ============================================================================
# Single Client Performance Tests
# ============================================================================


@pytest.mark.benchmark
class TestSingleClientPerformance:
    """Benchmark single client operations."""

    @pytest.mark.asyncio
    async def test_set_throughput(self, kv_client: KvClient):
        """
        Measure SET operations per second.

        This test measures raw write throughput with a single client.
        Target: >5,000 ops/sec
        """
        ops = 10000

        # Warm up
        for i in range(100):
            await kv_client.set(f"warmup:set:{i}", f"value{i}")

        # Benchmark
        start = time.perf_counter()
        for i in range(ops):
            await kv_client.set(f"bench:set:{i}", f"value{i}")
        elapsed = time.perf_counter() - start

        ops_per_sec = ops / elapsed
        print(f"\n[SET Throughput] {format_throughput(ops, elapsed)}")

        assert ops_per_sec > 5000, f"SET throughput too low: {ops_per_sec:.0f} ops/sec"

        # Cleanup
        for i in range(ops):
            await kv_client.delete(f"bench:set:{i}")

    @pytest.mark.asyncio
    async def test_get_throughput(self, kv_client: KvClient):
        """
        Measure GET operations per second.

        This test measures raw read throughput with pre-populated data.
        Target: >5,000 ops/sec
        """
        # Pre-populate 1000 keys
        dataset_size = 1000
        for i in range(dataset_size):
            await kv_client.set(f"bench:get:{i}", f"value{i}")

        ops = 10000

        # Warm up
        for i in range(100):
            await kv_client.get(f"bench:get:{i % dataset_size}")

        # Benchmark
        start = time.perf_counter()
        for i in range(ops):
            value = await kv_client.get(f"bench:get:{i % dataset_size}")
            assert value is not None
        elapsed = time.perf_counter() - start

        ops_per_sec = ops / elapsed
        print(f"\n[GET Throughput] {format_throughput(ops, elapsed)}")

        assert ops_per_sec > 5000, f"GET throughput too low: {ops_per_sec:.0f} ops/sec"

        # Cleanup
        for i in range(dataset_size):
            await kv_client.delete(f"bench:get:{i}")

    @pytest.mark.asyncio
    async def test_mixed_workload(self, kv_client: KvClient):
        """
        Measure mixed workload performance (50% GET, 50% SET).

        This simulates a realistic workload with both reads and writes.
        Target: >5,000 ops/sec
        """
        # Pre-populate 500 keys
        dataset_size = 500
        for i in range(dataset_size):
            await kv_client.set(f"bench:mixed:{i}", f"value{i}")

        ops = 10000

        # Warm up
        for i in range(100):
            if i % 2 == 0:
                await kv_client.get(f"bench:mixed:{i % dataset_size}")
            else:
                await kv_client.set(f"bench:mixed:{i % dataset_size}", f"new_value{i}")

        # Benchmark
        start = time.perf_counter()
        for i in range(ops):
            if i % 2 == 0:
                # GET
                value = await kv_client.get(f"bench:mixed:{i % dataset_size}")
            else:
                # SET
                await kv_client.set(f"bench:mixed:{i % dataset_size}", f"value{i}")
        elapsed = time.perf_counter() - start

        ops_per_sec = ops / elapsed
        print(f"\n[Mixed Workload] {format_throughput(ops, elapsed)}")

        assert ops_per_sec > 5000, f"Mixed workload throughput too low: {ops_per_sec:.0f} ops/sec"

        # Cleanup
        for i in range(dataset_size):
            await kv_client.delete(f"bench:mixed:{i}")

    @pytest.mark.asyncio
    async def test_incr_throughput(self, kv_client: KvClient):
        """
        Measure INCR operations per second.

        Tests atomic increment performance.
        Target: >5,000 ops/sec
        """
        # Initialize counter
        await kv_client.set("bench:counter", 0)

        ops = 5000

        # Warm up
        for _ in range(100):
            await kv_client.incr("bench:counter")

        # Benchmark
        start = time.perf_counter()
        for _ in range(ops):
            await kv_client.incr("bench:counter")
        elapsed = time.perf_counter() - start

        ops_per_sec = ops / elapsed
        print(f"\n[INCR Throughput] {format_throughput(ops, elapsed)}")

        assert ops_per_sec > 5000, f"INCR throughput too low: {ops_per_sec:.0f} ops/sec"

        # Verify final value
        final_value = await kv_client.get("bench:counter")
        assert final_value >= ops + 100  # ops + warmup

        # Cleanup
        await kv_client.delete("bench:counter")


# ============================================================================
# Connection Pool Performance Tests
# ============================================================================


@pytest.mark.benchmark
class TestPoolPerformance:
    """Benchmark connection pool operations."""

    @pytest.mark.asyncio
    async def test_pool_concurrent_set(self):
        """
        Measure pool performance with concurrent SET workers.

        This tests the pool's ability to handle concurrent operations
        across multiple workers.
        Target: >10,000 ops/sec with 10 workers
        """
        from tests.kv.conftest import KV_SERVER_ADDR

        pool = await KvPool.connect(
            PoolConfig(KV_SERVER_ADDR, min_size=5, max_size=20, idle_timeout=300.0)
        )

        async def worker(worker_id: int, ops_per_worker: int) -> None:
            """Worker function for concurrent operations."""
            for i in range(ops_per_worker):
                await pool.set(f"pool:set:{worker_id}:{i}", f"value{i}")

        workers = 10
        ops_per_worker = 1000
        total_ops = workers * ops_per_worker

        # Warm up
        await pool.set("warmup:pool", "value")

        # Benchmark
        start = time.perf_counter()
        await asyncio.gather(*[worker(w, ops_per_worker) for w in range(workers)])
        elapsed = time.perf_counter() - start

        ops_per_sec = total_ops / elapsed
        stats = await pool.stats()
        print(f"\n[Pool Concurrent SET] {format_throughput(total_ops, elapsed)}")
        print(f"Pool stats: {stats}")

        assert ops_per_sec > 10000, f"Pool throughput too low: {ops_per_sec:.0f} ops/sec"

        # Cleanup
        for w in range(workers):
            for i in range(ops_per_worker):
                await pool.delete(f"pool:set:{w}:{i}")

    @pytest.mark.asyncio
    async def test_pool_concurrent_mixed(self):
        """
        Measure pool performance with concurrent mixed workload.

        Tests pool with both reads and writes from multiple workers.
        Target: >10,000 ops/sec with 10 workers
        """
        from tests.kv.conftest import KV_SERVER_ADDR

        pool = await KvPool.connect(
            PoolConfig(KV_SERVER_ADDR, min_size=5, max_size=20, idle_timeout=300.0)
        )

        # Pre-populate
        dataset_size = 100
        for i in range(dataset_size):
            await pool.set(f"pool:mixed:{i}", f"value{i}")

        async def worker(worker_id: int, ops_per_worker: int) -> None:
            """Worker function with mixed operations."""
            for i in range(ops_per_worker):
                if i % 2 == 0:
                    await pool.get(f"pool:mixed:{i % dataset_size}")
                else:
                    await pool.set(f"pool:mixed:{i % dataset_size}", f"w{worker_id}:v{i}")

        workers = 10
        ops_per_worker = 1000
        total_ops = workers * ops_per_worker

        # Benchmark
        start = time.perf_counter()
        await asyncio.gather(*[worker(w, ops_per_worker) for w in range(workers)])
        elapsed = time.perf_counter() - start

        ops_per_sec = total_ops / elapsed
        stats = await pool.stats()
        print(f"\n[Pool Concurrent Mixed] {format_throughput(total_ops, elapsed)}")
        print(f"Pool stats: {stats}")

        assert ops_per_sec > 10000, f"Pool mixed throughput too low: {ops_per_sec:.0f} ops/sec"

        # Cleanup
        for i in range(dataset_size):
            await pool.delete(f"pool:mixed:{i}")

    @pytest.mark.asyncio
    async def test_pool_vs_single_client(self, kv_client: KvClient):
        """
        Compare pool performance vs single client for sequential operations.

        This measures the overhead of using a pool for sequential operations
        where connection reuse doesn't provide benefits.
        """
        from tests.kv.conftest import KV_SERVER_ADDR

        pool = await KvPool.connect(
            PoolConfig(KV_SERVER_ADDR, min_size=2, max_size=10, idle_timeout=300.0)
        )

        ops = 2000

        # Single client benchmark
        start = time.perf_counter()
        for i in range(ops):
            await kv_client.set(f"single:{i}", f"value{i}")
        single_elapsed = time.perf_counter() - start
        single_ops_per_sec = ops / single_elapsed

        # Pool benchmark
        start = time.perf_counter()
        for i in range(ops):
            await pool.set(f"pool:{i}", f"value{i}")
        pool_elapsed = time.perf_counter() - start
        pool_ops_per_sec = ops / pool_elapsed

        # Results
        overhead_percent = ((pool_elapsed - single_elapsed) / single_elapsed) * 100
        print(f"\n{'='*60}")
        print(f"Pool vs Single Client ({ops:,} sequential ops):")
        print(f"  Single:  {single_ops_per_sec:>10,.0f} ops/sec ({single_elapsed:.3f}s)")
        print(f"  Pool:    {pool_ops_per_sec:>10,.0f} ops/sec ({pool_elapsed:.3f}s)")
        print(f"  Overhead: {overhead_percent:>9.1f}%")
        print(f"{'='*60}")

        # Pool should have acceptable overhead (<20%)
        assert overhead_percent < 20, f"Pool overhead too high: {overhead_percent:.1f}%"

        # Cleanup
        for i in range(ops):
            await kv_client.delete(f"single:{i}")
            await pool.delete(f"pool:{i}")


# ============================================================================
# Redis Comparison Tests
# ============================================================================


@pytest.mark.benchmark
@pytest.mark.skipif(not REDIS_AVAILABLE, reason="redis-py not installed")
class TestRedisComparison:
    """Compare data-bridge KV performance with Redis."""

    @pytest.mark.asyncio
    async def test_set_comparison(self, kv_client: KvClient):
        """
        Compare SET performance: data-bridge vs Redis.

        This measures raw write performance of both systems.
        """
        # Connect to Redis
        r = redis.Redis(host="localhost", port=6379, decode_responses=True)

        try:
            # Ping Redis to ensure it's available
            await r.ping()
        except Exception as e:
            pytest.skip(f"Redis not available: {e}")

        ops = 5000

        # data-bridge benchmark
        start = time.perf_counter()
        for i in range(ops):
            await kv_client.set(f"cmp:db:{i}", f"value{i}")
        db_elapsed = time.perf_counter() - start
        db_ops_per_sec = ops / db_elapsed

        # Redis benchmark
        start = time.perf_counter()
        for i in range(ops):
            await r.set(f"cmp:redis:{i}", f"value{i}")
        redis_elapsed = time.perf_counter() - start
        redis_ops_per_sec = ops / redis_elapsed

        print_comparison("SET", db_ops_per_sec, redis_ops_per_sec, ops, db_elapsed, redis_elapsed)

        # Cleanup
        for i in range(ops):
            await kv_client.delete(f"cmp:db:{i}")
            await r.delete(f"cmp:redis:{i}")

        await r.aclose()

    @pytest.mark.asyncio
    async def test_get_comparison(self, kv_client: KvClient):
        """
        Compare GET performance: data-bridge vs Redis.

        This measures raw read performance of both systems.
        """
        # Connect to Redis
        r = redis.Redis(host="localhost", port=6379, decode_responses=True)

        try:
            await r.ping()
        except Exception as e:
            pytest.skip(f"Redis not available: {e}")

        # Pre-populate both systems
        dataset_size = 1000
        for i in range(dataset_size):
            await kv_client.set(f"cmp:db:get:{i}", f"value{i}")
            await r.set(f"cmp:redis:get:{i}", f"value{i}")

        ops = 10000

        # data-bridge benchmark
        start = time.perf_counter()
        for i in range(ops):
            value = await kv_client.get(f"cmp:db:get:{i % dataset_size}")
            assert value is not None
        db_elapsed = time.perf_counter() - start
        db_ops_per_sec = ops / db_elapsed

        # Redis benchmark
        start = time.perf_counter()
        for i in range(ops):
            value = await r.get(f"cmp:redis:get:{i % dataset_size}")
            assert value is not None
        redis_elapsed = time.perf_counter() - start
        redis_ops_per_sec = ops / redis_elapsed

        print_comparison("GET", db_ops_per_sec, redis_ops_per_sec, ops, db_elapsed, redis_elapsed)

        # Cleanup
        for i in range(dataset_size):
            await kv_client.delete(f"cmp:db:get:{i}")
            await r.delete(f"cmp:redis:get:{i}")

        await r.aclose()

    @pytest.mark.asyncio
    async def test_mixed_comparison(self, kv_client: KvClient):
        """
        Compare mixed workload: data-bridge vs Redis.

        50% GET, 50% SET operations.
        """
        # Connect to Redis
        r = redis.Redis(host="localhost", port=6379, decode_responses=True)

        try:
            await r.ping()
        except Exception as e:
            pytest.skip(f"Redis not available: {e}")

        # Pre-populate
        dataset_size = 500
        for i in range(dataset_size):
            await kv_client.set(f"cmp:db:mixed:{i}", f"value{i}")
            await r.set(f"cmp:redis:mixed:{i}", f"value{i}")

        ops = 5000

        # data-bridge benchmark
        start = time.perf_counter()
        for i in range(ops):
            if i % 2 == 0:
                await kv_client.get(f"cmp:db:mixed:{i % dataset_size}")
            else:
                await kv_client.set(f"cmp:db:mixed:{i % dataset_size}", f"new{i}")
        db_elapsed = time.perf_counter() - start
        db_ops_per_sec = ops / db_elapsed

        # Redis benchmark
        start = time.perf_counter()
        for i in range(ops):
            if i % 2 == 0:
                await r.get(f"cmp:redis:mixed:{i % dataset_size}")
            else:
                await r.set(f"cmp:redis:mixed:{i % dataset_size}", f"new{i}")
        redis_elapsed = time.perf_counter() - start
        redis_ops_per_sec = ops / redis_elapsed

        print_comparison(
            "Mixed (50% GET, 50% SET)",
            db_ops_per_sec,
            redis_ops_per_sec,
            ops,
            db_elapsed,
            redis_elapsed,
        )

        # Cleanup
        for i in range(dataset_size):
            await kv_client.delete(f"cmp:db:mixed:{i}")
            await r.delete(f"cmp:redis:mixed:{i}")

        await r.aclose()

    @pytest.mark.asyncio
    async def test_incr_comparison(self, kv_client: KvClient):
        """
        Compare INCR performance: data-bridge vs Redis.

        Tests atomic increment operations.
        """
        # Connect to Redis
        r = redis.Redis(host="localhost", port=6379, decode_responses=True)

        try:
            await r.ping()
        except Exception as e:
            pytest.skip(f"Redis not available: {e}")

        # Initialize counters
        await kv_client.set("cmp:db:counter", 0)
        await r.set("cmp:redis:counter", 0)

        ops = 5000

        # data-bridge benchmark
        start = time.perf_counter()
        for _ in range(ops):
            await kv_client.incr("cmp:db:counter")
        db_elapsed = time.perf_counter() - start
        db_ops_per_sec = ops / db_elapsed

        # Redis benchmark
        start = time.perf_counter()
        for _ in range(ops):
            await r.incr("cmp:redis:counter")
        redis_elapsed = time.perf_counter() - start
        redis_ops_per_sec = ops / redis_elapsed

        print_comparison("INCR", db_ops_per_sec, redis_ops_per_sec, ops, db_elapsed, redis_elapsed)

        # Verify values
        db_value = await kv_client.get("cmp:db:counter")
        redis_value = await r.get("cmp:redis:counter")
        assert db_value == ops
        assert int(redis_value) == ops

        # Cleanup
        await kv_client.delete("cmp:db:counter")
        await r.delete("cmp:redis:counter")

        await r.aclose()


# ============================================================================
# Latency Distribution Tests
# ============================================================================


@pytest.mark.benchmark
class TestLatencyDistribution:
    """Measure latency distribution for operations."""

    @pytest.mark.asyncio
    async def test_set_latency_distribution(self, kv_client: KvClient):
        """
        Measure latency distribution for SET operations.

        Records individual operation latencies to compute percentiles.
        """
        ops = 1000
        latencies: List[float] = []

        # Benchmark with per-operation timing
        for i in range(ops):
            start = time.perf_counter()
            await kv_client.set(f"latency:set:{i}", f"value{i}")
            elapsed = time.perf_counter() - start
            latencies.append(elapsed * 1000)  # Convert to milliseconds

        # Calculate statistics
        latencies.sort()
        p50 = latencies[len(latencies) // 2]
        p95 = latencies[int(len(latencies) * 0.95)]
        p99 = latencies[int(len(latencies) * 0.99)]
        avg = sum(latencies) / len(latencies)
        min_lat = min(latencies)
        max_lat = max(latencies)

        print(f"\n{'='*60}")
        print(f"SET Latency Distribution ({ops} ops):")
        print(f"  Min:    {min_lat:>8.3f} ms")
        print(f"  Avg:    {avg:>8.3f} ms")
        print(f"  P50:    {p50:>8.3f} ms")
        print(f"  P95:    {p95:>8.3f} ms")
        print(f"  P99:    {p99:>8.3f} ms")
        print(f"  Max:    {max_lat:>8.3f} ms")
        print(f"{'='*60}")

        # Assertions (reasonable latency expectations)
        assert avg < 1.0, f"Average latency too high: {avg:.3f}ms"
        assert p99 < 5.0, f"P99 latency too high: {p99:.3f}ms"

        # Cleanup
        for i in range(ops):
            await kv_client.delete(f"latency:set:{i}")

    @pytest.mark.asyncio
    async def test_get_latency_distribution(self, kv_client: KvClient):
        """
        Measure latency distribution for GET operations.

        Records individual operation latencies to compute percentiles.
        """
        # Pre-populate
        dataset_size = 100
        for i in range(dataset_size):
            await kv_client.set(f"latency:get:{i}", f"value{i}")

        ops = 1000
        latencies: List[float] = []

        # Benchmark with per-operation timing
        for i in range(ops):
            start = time.perf_counter()
            value = await kv_client.get(f"latency:get:{i % dataset_size}")
            elapsed = time.perf_counter() - start
            latencies.append(elapsed * 1000)  # Convert to milliseconds
            assert value is not None

        # Calculate statistics
        latencies.sort()
        p50 = latencies[len(latencies) // 2]
        p95 = latencies[int(len(latencies) * 0.95)]
        p99 = latencies[int(len(latencies) * 0.99)]
        avg = sum(latencies) / len(latencies)
        min_lat = min(latencies)
        max_lat = max(latencies)

        print(f"\n{'='*60}")
        print(f"GET Latency Distribution ({ops} ops):")
        print(f"  Min:    {min_lat:>8.3f} ms")
        print(f"  Avg:    {avg:>8.3f} ms")
        print(f"  P50:    {p50:>8.3f} ms")
        print(f"  P95:    {p95:>8.3f} ms")
        print(f"  P99:    {p99:>8.3f} ms")
        print(f"  Max:    {max_lat:>8.3f} ms")
        print(f"{'='*60}")

        # Assertions
        assert avg < 1.0, f"Average latency too high: {avg:.3f}ms"
        assert p99 < 5.0, f"P99 latency too high: {p99:.3f}ms"

        # Cleanup
        for i in range(dataset_size):
            await kv_client.delete(f"latency:get:{i}")
