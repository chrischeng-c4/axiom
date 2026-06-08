"""Tests for run_forever and run_until_complete."""

import time
import pytest
from cclab.orbit import PyLoop


class TestRunForever:
    """Tests for run_forever method."""

    def test_run_forever_with_stop(self):
        """Test that run_forever stops when stop() is called."""
        loop = PyLoop()
        called = []

        def callback():
            called.append(1)
            loop.stop()

        loop.call_soon(callback)
        loop.run_forever()

        assert len(called) == 1
        assert not loop.is_running()

    def test_run_forever_on_closed_loop(self):
        """Test that run_forever fails on closed loop."""
        loop = PyLoop()
        loop.close()

        with pytest.raises(RuntimeError, match="Event loop is closed"):
            loop.run_forever()

    def test_run_forever_already_running(self):
        """Test that run_forever fails if loop is already running."""
        loop = PyLoop()

        def try_run_again():
            with pytest.raises(RuntimeError, match="already running"):
                loop.run_forever()
            loop.stop()

        loop.call_soon(try_run_again)
        loop.run_forever()

    def test_run_forever_processes_multiple_callbacks(self):
        """Test that run_forever processes multiple callbacks."""
        loop = PyLoop()
        results = []

        def callback1():
            results.append(1)

        def callback2():
            results.append(2)

        def callback3():
            results.append(3)
            loop.stop()

        loop.call_soon(callback1)
        loop.call_soon(callback2)
        loop.call_soon(callback3)

        loop.run_forever()

        assert results == [1, 2, 3]

    def test_run_forever_with_delayed_stop(self):
        """Test that run_forever stops after a delayed callback."""
        loop = PyLoop()
        called = []

        def callback():
            called.append(time.time())
            loop.stop()

        start = time.time()
        loop.call_later(0.1, callback)
        loop.run_forever()
        elapsed = time.time() - start

        assert len(called) == 1
        assert elapsed >= 0.1  # At least 100ms delay
        assert not loop.is_running()

    def test_run_forever_with_exception_in_callback(self):
        """Test that run_forever continues after exception in callback."""
        loop = PyLoop()
        results = []

        def bad_callback():
            results.append('bad')
            raise ValueError("Test exception")

        def good_callback():
            results.append('good')
            loop.stop()

        loop.call_soon(bad_callback)
        loop.call_soon(good_callback)

        # Should not raise, exception is printed
        loop.run_forever()

        assert results == ['bad', 'good']


class TestRunUntilComplete:
    """Tests for run_until_complete method."""

    @pytest.mark.skip(reason="Coroutine execution not fully implemented in Phase 2.5")
    def test_run_until_complete_with_simple_coroutine(self):
        """Test run_until_complete with a simple coroutine."""
        loop = PyLoop()

        async def my_coro():
            return 42

        result = loop.run_until_complete(my_coro())
        assert result == 42

    @pytest.mark.skip(reason="Coroutine execution not fully implemented in Phase 2.5")
    def test_run_until_complete_with_task(self):
        """Test run_until_complete with a Task."""
        loop = PyLoop()

        async def my_coro():
            return 99

        task = loop.create_task(my_coro())
        result = loop.run_until_complete(task)

        assert result == 99

    def test_run_until_complete_not_coroutine(self):
        """Test that run_until_complete requires coroutine or Task."""
        loop = PyLoop()

        with pytest.raises(TypeError, match="requires a coroutine or Task"):
            loop.run_until_complete(42)

    def test_run_until_complete_on_closed_loop(self):
        """Test that run_until_complete fails on closed loop."""
        loop = PyLoop()
        loop.close()

        async def my_coro():
            return 42

        with pytest.raises(RuntimeError, match="Event loop is closed"):
            loop.run_until_complete(my_coro())

    def test_run_until_complete_already_running(self):
        """Test that run_until_complete fails if loop is already running."""
        loop = PyLoop()

        async def inner_coro():
            return 123

        def try_run_again():
            with pytest.raises(RuntimeError, match="already running"):
                loop.run_until_complete(inner_coro())
            loop.stop()

        loop.call_soon(try_run_again)
        loop.run_forever()

    @pytest.mark.skip(reason="Coroutine execution not fully implemented in Phase 2.5")
    def test_run_until_complete_with_none_return(self):
        """Test run_until_complete with coroutine returning None."""
        loop = PyLoop()

        async def my_coro():
            pass  # Returns None implicitly

        result = loop.run_until_complete(my_coro())
        assert result is None


class TestStop:
    """Tests for stop method."""

    def test_stop_when_not_running(self):
        """Test that stop() is safe when loop is not running."""
        loop = PyLoop()
        loop.stop()  # Should not raise

    def test_stop_from_callback(self):
        """Test stopping loop from within a callback."""
        loop = PyLoop()
        stopped = []

        def callback():
            stopped.append(True)
            loop.stop()

        loop.call_soon(callback)
        loop.run_forever()

        assert stopped == [True]

    def test_stop_from_delayed_callback(self):
        """Test stopping loop from delayed callback."""
        loop = PyLoop()
        stopped = []

        def callback():
            stopped.append(True)
            loop.stop()

        start = time.time()
        loop.call_later(0.1, callback)
        loop.run_forever()
        elapsed = time.time() - start

        assert stopped == [True]
        assert elapsed >= 0.1


class TestLoopInteraction:
    """Tests for interactions between methods."""

    def test_call_soon_during_run_forever(self):
        """Test that callbacks scheduled during run_forever are executed."""
        loop = PyLoop()
        results = []

        def callback1():
            results.append(1)
            # Schedule another callback while loop is running
            loop.call_soon(callback2)

        def callback2():
            results.append(2)
            loop.stop()

        loop.call_soon(callback1)
        loop.run_forever()

        assert results == [1, 2]

    def test_call_later_during_run_forever(self):
        """Test that delayed callbacks scheduled during run_forever are executed."""
        loop = PyLoop()
        results = []

        def callback1():
            results.append(1)
            # Schedule delayed callback while loop is running
            loop.call_later(0.05, callback2)

        def callback2():
            results.append(2)
            loop.stop()

        start = time.time()
        loop.call_soon(callback1)
        loop.run_forever()
        elapsed = time.time() - start

        assert results == [1, 2]
        assert elapsed >= 0.05


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
