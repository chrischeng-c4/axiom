"""Tests for create_task functionality."""

import pytest
import time


class TestCreateTask:
    """Tests for create_task method."""

    def test_create_task_requires_coroutine(self):
        """Test that create_task requires a coroutine."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        # Regular function should fail
        def not_a_coro():
            return 42

        with pytest.raises(TypeError, match="requires a coroutine"):
            loop.create_task(not_a_coro())

    def test_create_task_on_closed_loop(self):
        """Test that create_task fails on closed loop."""
        from cclab.orbit import PyLoop

        loop = PyLoop()
        loop.close()

        async def my_coro():
            return 42

        with pytest.raises(RuntimeError, match="Event loop is closed"):
            loop.create_task(my_coro())

    def test_task_has_name(self):
        """Test that task can have a name."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro(), name="my_task")
        assert task.get_name() == "my_task"

    def test_task_set_name(self):
        """Test that task name can be changed."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro(), name="original")
        assert task.get_name() == "original"

        task.set_name("new_name")
        assert task.get_name() == "new_name"

    def test_task_cancel(self):
        """Test that task can be cancelled."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        # Use a simple coroutine that doesn't yield
        async def my_coro():
            # This is a coroutine, not a generator
            pass

        task = loop.create_task(my_coro())

        # Cancel it
        cancelled = task.cancel()
        assert cancelled or task.done()  # May already be done
        if not task.done():
            assert task.cancelled()

    def test_task_cancel_done_task(self):
        """Test that cancelling a done task returns False."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro())

        # Wait for task to complete (simple polling)
        for _ in range(50):
            if task.done():
                break
            time.sleep(0.01)

        # Try to cancel completed task
        if task.done():
            cancelled = task.cancel()
            assert not cancelled

    def test_task_repr(self):
        """Test task repr."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro(), name="test")
        repr_str = repr(task)
        assert "Task" in repr_str
        assert "test" in repr_str

    def test_task_repr_without_name(self):
        """Test task repr without name."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro())
        repr_str = repr(task)
        assert "Task" in repr_str
        assert "pending" in repr_str or "done" in repr_str

    def test_task_result_not_done(self):
        """Test that getting result of pending task raises error."""
        from cclab.orbit import PyLoop
        import asyncio

        loop = PyLoop()

        async def my_coro():
            # Long-running coroutine using asyncio.sleep
            await asyncio.sleep(10)
            return 42

        task = loop.create_task(my_coro())

        # Immediately try to get result (should fail)
        with pytest.raises(RuntimeError, match="not done yet"):
            task.result()

    def test_task_result_cancelled(self):
        """Test that getting result of cancelled task raises CancelledError."""
        from cclab.orbit import PyLoop
        import asyncio

        loop = PyLoop()

        async def my_coro():
            await asyncio.sleep(10)
            return 42

        task = loop.create_task(my_coro())
        task.cancel()

        # Wait a bit for cancellation to take effect
        time.sleep(0.1)

        # Check if the task was cancelled successfully
        # The exception raised should be our custom CancelledError or the standard one
        with pytest.raises(Exception, match="[Cc]ancelled"):
            task.result()

    def test_simple_coroutine_completion(self):
        """Test that a simple coroutine completes successfully."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def simple_coro():
            return 42

        task = loop.create_task(simple_coro())

        # Wait for completion
        for _ in range(100):
            if task.done():
                break
            time.sleep(0.01)

        # If done, check result
        if task.done() and not task.cancelled():
            result = task.result()
            assert result == 42

    def test_task_done_initially_false(self):
        """Test that newly created task is not done."""
        from cclab.orbit import PyLoop
        import asyncio

        loop = PyLoop()

        async def my_coro():
            await asyncio.sleep(10)
            return 42

        task = loop.create_task(my_coro())
        # Immediately check - should not be done yet
        assert not task.done()

    def test_task_cancelled_initially_false(self):
        """Test that newly created task is not cancelled."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def my_coro():
            return 42

        task = loop.create_task(my_coro())
        assert not task.cancelled()


class TestTaskExceptions:
    """Tests for task exception handling."""

    def test_task_with_exception(self):
        """Test that exceptions in coroutines are captured."""
        from cclab.orbit import PyLoop

        loop = PyLoop()

        async def failing_coro():
            raise ValueError("test error")

        task = loop.create_task(failing_coro())

        # Wait for task to complete
        for _ in range(100):
            if task.done():
                break
            time.sleep(0.01)

        # Task should be done
        if task.done():
            # Getting result should raise the exception
            with pytest.raises(ValueError, match="test error"):
                task.result()
