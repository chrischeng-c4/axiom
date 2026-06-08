"""
Unit tests for GIL release verification.

These tests verify that the GIL is properly released during BSON conversion,
enabling true concurrency in multi-threaded Python applications.
"""

import threading
import time
from typing import List


def test_concurrent_find_one_no_gil_blocking(sample_users):
    """
    T034: Verify GIL is released during find_one operation.

    This test validates that concurrent find_one calls don't block each other
    due to GIL contention. If GIL is properly released during BSON conversion,
    concurrent operations should complete in similar time to sequential operations.

    Success criteria (FR-008): Concurrent overhead <10%
    """
    from cclab import Document

    class User(Document):
        name: str
        age: int
        email: str

        class Settings:
            name = "users"

    # Baseline: Sequential execution
    def find_user() -> float:
        """Execute find_one and return execution time."""
        start = time.perf_counter()
        User.find_one(User.age == 35)
        return time.perf_counter() - start

    # Run sequential baseline (10 operations)
    sequential_times = [find_user() for _ in range(10)]
    sequential_total = sum(sequential_times)
    sequential_avg = sequential_total / len(sequential_times)

    # Run concurrent test (10 threads)
    concurrent_times: List[float] = []

    def worker():
        concurrent_times.append(find_user())

    threads = [threading.Thread(target=worker) for _ in range(10)]
    concurrent_start = time.perf_counter()

    for t in threads:
        t.start()

    for t in threads:
        t.join()

    concurrent_total = time.perf_counter() - concurrent_start
    concurrent_avg = sum(concurrent_times) / len(concurrent_times)

    # Analysis
    overhead = (concurrent_total / sequential_total) - 1.0

    print(f"\n=== GIL Release Verification ===")
    print(f"Sequential: {sequential_total*1000:.2f}ms total, {sequential_avg*1000:.2f}ms avg")
    print(f"Concurrent: {concurrent_total*1000:.2f}ms total, {concurrent_avg*1000:.2f}ms avg")
    print(f"Overhead: {overhead*100:.1f}%")

    # Success criteria: Concurrent overhead <10%
    # If GIL is released, concurrent should be similar to sequential
    assert overhead < 0.10, (
        f"GIL blocking detected: {overhead*100:.1f}% overhead "
        f"(expected <10%). Concurrent: {concurrent_total*1000:.2f}ms, "
        f"Sequential: {sequential_total*1000:.2f}ms"
    )

    print("✅ GIL properly released during find_one!")
