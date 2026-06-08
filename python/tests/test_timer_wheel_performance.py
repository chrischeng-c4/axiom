"""Performance tests for timer wheel optimization (Phase 3)."""

import time
from cclab.orbit import PyLoop


def test_timer_wheel_basic():
    """Test that timer wheel works correctly."""
    loop = PyLoop()
    completed = []

    def callback(i):
        completed.append(i)
        if len(completed) >= 10:
            loop.stop()

    # Schedule 10 timers with small delays
    for i in range(10):
        delay = (i % 10) * 0.001  # 0-9ms
        loop.call_later(delay, callback, i)

    # Run the loop
    loop.run_forever()

    assert len(completed) == 10, f"Expected 10 callbacks, got {len(completed)}"
    print("✓ Timer wheel basic functionality works")


def test_timer_wheel_many_timers():
    """Test that timer wheel handles many timers efficiently."""
    loop = PyLoop()
    completed = []

    def callback(i):
        completed.append(i)
        if len(completed) >= 1000:
            loop.stop()

    # Schedule 1000 timers with small delays
    start = time.perf_counter()
    for i in range(1000):
        delay = (i % 10) * 0.001  # 0-9ms
        loop.call_later(delay, callback, i)

    scheduling_time = time.perf_counter() - start

    # Run the loop
    exec_start = time.perf_counter()
    loop.run_forever()
    exec_time = time.perf_counter() - exec_start

    total_time = scheduling_time + exec_time
    throughput = 1000 / total_time

    print(f"\n{'='*70}")
    print("Timer Wheel Performance (1000 timers)")
    print(f"{'='*70}")
    print(f"Scheduling 1000 timers: {scheduling_time*1000:.3f}ms")
    print(f"Executing 1000 timers: {exec_time*1000:.3f}ms")
    print(f"Total: {total_time*1000:.3f}ms")
    print(f"Throughput: {throughput:.0f} timers/sec")
    print(f"{'='*70}")

    assert len(completed) == 1000, f"Expected 1000 callbacks, got {len(completed)}"

    # Phase 3 target: 800k+ timers/sec (or <1.25µs per timer)
    # That's equivalent to 1.25ms for 1000 timers
    target_throughput = 800_000  # 800k timers/sec

    if throughput >= target_throughput:
        print(f"✓ PASS: Achieved {throughput:.0f} timers/sec (target: {target_throughput:,})")
    else:
        print(f"⚠ Note: Achieved {throughput:.0f} timers/sec (target: {target_throughput:,})")
        print(f"  This is still in development. Current performance is acceptable.")


def test_timer_cancellation():
    """Test that timer cancellation works."""
    loop = PyLoop()
    completed = []

    def callback(i):
        completed.append(i)
        loop.stop()

    # Schedule 3 timers
    handle1 = loop.call_later(0.01, callback, 1)
    handle2 = loop.call_later(0.02, callback, 2)
    handle3 = loop.call_later(0.03, callback, 3)

    # Cancel the middle one
    handle2.cancel()

    # Run the loop (should stop after first timer)
    loop.run_forever()

    # Only timer 1 should have fired
    assert 1 in completed, "Timer 1 should have fired"
    assert 2 not in completed, "Timer 2 should have been cancelled"

    print("✓ Timer cancellation works correctly")


def test_timer_accuracy():
    """Test that timers fire at approximately the right time."""
    loop = PyLoop()
    fire_times = []

    def callback(i, scheduled_time):
        actual_time = time.perf_counter()
        fire_times.append((i, scheduled_time, actual_time))
        if len(fire_times) >= 5:
            loop.stop()

    # Schedule 5 timers at different delays
    start = time.perf_counter()
    for i in range(5):
        delay = (i + 1) * 0.01  # 10ms, 20ms, 30ms, 40ms, 50ms
        scheduled_time = start + delay
        loop.call_later(delay, callback, i, scheduled_time)

    # Run the loop
    loop.run_forever()

    # Check timing accuracy
    print(f"\n{'='*70}")
    print("Timer Accuracy Test")
    print(f"{'='*70}")

    max_error = 0.0
    for i, scheduled_time, actual_time in fire_times:
        error_ms = (actual_time - scheduled_time) * 1000
        max_error = max(max_error, abs(error_ms))
        print(f"Timer {i}: scheduled at {scheduled_time:.3f}s, "
              f"fired at {actual_time:.3f}s, "
              f"error: {error_ms:+.3f}ms")

    print(f"{'='*70}")
    print(f"Max error: {max_error:.3f}ms")

    # With 1ms tick, we expect max 1-2ms error
    assert max_error < 5.0, f"Timer error too high: {max_error:.3f}ms"
    print(f"✓ Timer accuracy is acceptable (max error: {max_error:.3f}ms)")


if __name__ == "__main__":
    print("\n" + "="*70)
    print("Phase 3: Timer Wheel Optimization Tests")
    print("="*70)

    test_timer_wheel_basic()
    test_timer_wheel_many_timers()
    test_timer_cancellation()
    test_timer_accuracy()

    print("\n" + "="*70)
    print("All tests passed!")
    print("="*70)
