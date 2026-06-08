"""Tests for call_later and call_at timing functionality."""

import time
import pytest
from cclab.orbit import PyLoop


class TestCallLater:
    """Tests for call_later method."""

    def test_call_later_basic(self):
        """Test that call_later schedules a callback after delay."""
        loop = PyLoop()
        called = []

        def callback(value):
            called.append(value)

        handle = loop.call_later(0.1, callback, 42)
        assert not handle.cancelled()
        # Check that it's a different type from regular Handle
        assert "TimerHandle" in type(handle).__name__

    def test_call_later_zero_delay(self):
        """Test that zero delay is valid."""
        loop = PyLoop()

        handle = loop.call_later(0.0, lambda: None)
        assert not handle.cancelled()

    def test_call_later_negative_delay_raises(self):
        """Test that negative delay raises ValueError."""
        loop = PyLoop()

        with pytest.raises(ValueError, match="delay must be non-negative"):
            loop.call_later(-1.0, lambda: None)

    def test_call_later_on_closed_loop(self):
        """Test that call_later on closed loop raises."""
        loop = PyLoop()
        loop.close()

        with pytest.raises(RuntimeError, match="Event loop is closed"):
            loop.call_later(1.0, lambda: None)

    def test_timer_handle_cancel(self):
        """Test that TimerHandle can be cancelled."""
        loop = PyLoop()

        handle = loop.call_later(10.0, lambda: None)
        assert not handle.cancelled()

        handle.cancel()
        assert handle.cancelled()

    def test_timer_handle_with_args(self):
        """Test that call_later passes arguments correctly."""
        loop = PyLoop()

        # Just verify it accepts arguments, not that they're called
        # (that would require running the loop)
        handle = loop.call_later(0.1, print, "arg1", "arg2", "arg3")
        assert not handle.cancelled()


class TestCallAt:
    """Tests for call_at method."""

    def test_call_at_basic(self):
        """Test that call_at schedules a callback at absolute time."""
        loop = PyLoop()

        # Schedule for "1 second from now" in loop time
        when = loop.time() + 1.0
        handle = loop.call_at(when, lambda: None)

        assert not handle.cancelled()
        assert "TimerHandle" in type(handle).__name__

    def test_call_at_past_time(self):
        """Test that call_at with past time schedules immediately."""
        loop = PyLoop()

        # Schedule for a time in the past
        when = loop.time() - 1.0
        handle = loop.call_at(when, lambda: None)

        # Should not raise, will schedule with 0 delay
        assert not handle.cancelled()

    def test_call_at_current_time(self):
        """Test that call_at at current time works."""
        loop = PyLoop()

        when = loop.time()
        handle = loop.call_at(when, lambda: None)

        assert not handle.cancelled()

    def test_call_at_with_args(self):
        """Test that call_at passes arguments correctly."""
        loop = PyLoop()

        when = loop.time() + 0.1
        handle = loop.call_at(when, print, "test", 123)
        assert not handle.cancelled()

    def test_call_at_on_closed_loop(self):
        """Test that call_at on closed loop raises."""
        loop = PyLoop()
        loop.close()

        with pytest.raises(RuntimeError, match="Event loop is closed"):
            loop.call_at(1.0, lambda: None)


class TestLoopTime:
    """Tests for loop.time() method."""

    def test_loop_time_progresses(self):
        """Test that loop.time() returns increasing values."""
        loop = PyLoop()

        time1 = loop.time()
        time.sleep(0.01)
        time2 = loop.time()

        assert time2 >= time1
        # Should have advanced by roughly 10ms (0.01s)
        assert time2 - time1 >= 0.009  # Allow some tolerance

    def test_loop_time_starts_at_zero(self):
        """Test that loop time starts near zero on first call."""
        loop = PyLoop()

        time1 = loop.time()
        # Should be very close to 0 on first call
        assert time1 >= 0.0
        assert time1 < 0.1  # Within 100ms

    def test_loop_time_consistent_reference(self):
        """Test that multiple loops have independent time references."""
        loop1 = PyLoop()
        loop2 = PyLoop()

        time1_a = loop1.time()
        time.sleep(0.01)
        time2_a = loop2.time()

        # Each loop should start from near zero
        assert time1_a < 0.1
        assert time2_a < 0.1

    def test_loop_time_on_closed_loop(self):
        """Test that time() works even on closed loop."""
        loop = PyLoop()

        time1 = loop.time()
        loop.close()
        time2 = loop.time()

        # Should still work
        assert time2 >= time1


class TestTimerHandleRepr:
    """Tests for timer handle repr."""

    def test_timer_handle_repr_active(self):
        """Test TimerHandle repr when active."""
        loop = PyLoop()
        handle = loop.call_later(1.0, lambda: None)

        repr_str = repr(handle)
        assert "TimerHandle" in repr_str
        assert "active" in repr_str

    def test_timer_handle_repr_cancelled(self):
        """Test TimerHandle repr when cancelled."""
        loop = PyLoop()
        handle = loop.call_later(1.0, lambda: None)
        handle.cancel()

        repr_str = repr(handle)
        assert "TimerHandle" in repr_str
        assert "cancelled" in repr_str


class TestTimerHandleType:
    """Tests for TimerHandle type behavior."""

    def test_timer_handle_is_different_from_handle(self):
        """Test that TimerHandle and Handle are different types."""
        loop = PyLoop()

        regular_handle = loop.call_soon(lambda: None)
        timer_handle = loop.call_later(1.0, lambda: None)

        # They should have different type names
        assert type(regular_handle).__name__ == "Handle"
        assert type(timer_handle).__name__ == "TimerHandle"

    def test_timer_handle_cancel_is_safe(self):
        """Test that cancelling a timer handle multiple times is safe."""
        loop = PyLoop()
        handle = loop.call_later(1.0, lambda: None)

        # Cancel multiple times should be safe
        handle.cancel()
        handle.cancel()
        handle.cancel()

        assert handle.cancelled()


class TestCallLaterCallAtEdgeCases:
    """Edge cases for call_later and call_at."""

    def test_call_later_very_large_delay(self):
        """Test call_later with very large delay."""
        loop = PyLoop()

        # 1 year in seconds
        handle = loop.call_later(365 * 24 * 3600, lambda: None)
        assert not handle.cancelled()

        # Should be cancellable
        handle.cancel()
        assert handle.cancelled()

    def test_call_at_very_far_future(self):
        """Test call_at with very far future time."""
        loop = PyLoop()

        # 1 year from now
        when = loop.time() + (365 * 24 * 3600)
        handle = loop.call_at(when, lambda: None)
        assert not handle.cancelled()

    def test_call_later_fractional_seconds(self):
        """Test call_later with fractional seconds."""
        loop = PyLoop()

        # Test various fractional delays
        for delay in [0.001, 0.1, 0.5, 1.5, 2.999]:
            handle = loop.call_later(delay, lambda: None)
            assert not handle.cancelled()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
