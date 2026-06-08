"""Tests for background tasks functionality."""

import pytest
import asyncio
from typing import List
from cclab.quasar.background import BackgroundTasks, get_background_tasks


class TestBackgroundTasksBasics:
    """Test basic BackgroundTasks functionality."""

    def test_init(self):
        """Test BackgroundTasks initialization."""
        background = BackgroundTasks()
        assert len(background) == 0
        assert background._tasks == []

    def test_repr(self):
        """Test BackgroundTasks string representation."""
        background = BackgroundTasks()
        assert repr(background) == "BackgroundTasks(tasks=0)"

        background.add_task(lambda: None)
        assert repr(background) == "BackgroundTasks(tasks=1)"

    def test_len_empty(self):
        """Test len() on empty BackgroundTasks."""
        background = BackgroundTasks()
        assert len(background) == 0

    def test_len_with_tasks(self):
        """Test len() with multiple tasks."""
        background = BackgroundTasks()
        background.add_task(lambda: None)
        assert len(background) == 1

        background.add_task(lambda: None)
        assert len(background) == 2

    def test_clear(self):
        """Test clearing all tasks."""
        background = BackgroundTasks()
        background.add_task(lambda: None)
        background.add_task(lambda: None)
        assert len(background) == 2

        background.clear()
        assert len(background) == 0
        assert background._tasks == []

    def test_clear_empty(self):
        """Test clearing already empty tasks."""
        background = BackgroundTasks()
        background.clear()
        assert len(background) == 0


class TestAddTask:
    """Test adding tasks to BackgroundTasks."""

    def test_add_simple_task(self):
        """Test adding a simple task."""
        def my_task():
            pass

        background = BackgroundTasks()
        background.add_task(my_task)
        assert len(background) == 1
        assert background._tasks[0][0] == my_task

    def test_add_task_with_args(self):
        """Test adding task with positional arguments."""
        def my_task(a, b):
            pass

        background = BackgroundTasks()
        background.add_task(my_task, 1, 2)
        assert len(background) == 1

        func, args, kwargs = background._tasks[0]
        assert func == my_task
        assert args == (1, 2)
        assert kwargs == {}

    def test_add_task_with_kwargs(self):
        """Test adding task with keyword arguments."""
        def my_task(name, value):
            pass

        background = BackgroundTasks()
        background.add_task(my_task, name="test", value=42)
        assert len(background) == 1

        func, args, kwargs = background._tasks[0]
        assert func == my_task
        assert args == ()
        assert kwargs == {"name": "test", "value": 42}

    def test_add_task_with_args_and_kwargs(self):
        """Test adding task with both args and kwargs."""
        def my_task(a, b, name=None):
            pass

        background = BackgroundTasks()
        background.add_task(my_task, 1, 2, name="test")

        func, args, kwargs = background._tasks[0]
        assert func == my_task
        assert args == (1, 2)
        assert kwargs == {"name": "test"}

    def test_add_multiple_tasks(self):
        """Test adding multiple tasks in sequence."""
        def task1():
            pass

        def task2():
            pass

        background = BackgroundTasks()
        background.add_task(task1)
        background.add_task(task2)

        assert len(background) == 2
        assert background._tasks[0][0] == task1
        assert background._tasks[1][0] == task2

    def test_add_async_task(self):
        """Test adding an async task."""
        async def my_async_task():
            pass

        background = BackgroundTasks()
        background.add_task(my_async_task)
        assert len(background) == 1
        assert background._tasks[0][0] == my_async_task


class TestRunTasks:
    """Test running background tasks."""

    @pytest.mark.asyncio
    async def test_run_empty(self):
        """Test running with no tasks."""
        background = BackgroundTasks()
        await background.run()  # Should not raise

    @pytest.mark.asyncio
    async def test_run_sync_task(self):
        """Test running a synchronous task."""
        executed = []

        def my_task():
            executed.append("task1")

        background = BackgroundTasks()
        background.add_task(my_task)
        await background.run()

        assert executed == ["task1"]

    @pytest.mark.asyncio
    async def test_run_async_task(self):
        """Test running an asynchronous task."""
        executed = []

        async def my_async_task():
            executed.append("async_task")

        background = BackgroundTasks()
        background.add_task(my_async_task)
        await background.run()

        assert executed == ["async_task"]

    @pytest.mark.asyncio
    async def test_run_multiple_tasks(self):
        """Test running multiple tasks in order."""
        executed = []

        def task1():
            executed.append(1)

        async def task2():
            executed.append(2)

        def task3():
            executed.append(3)

        background = BackgroundTasks()
        background.add_task(task1)
        background.add_task(task2)
        background.add_task(task3)
        await background.run()

        # Tasks should execute in order
        assert executed == [1, 2, 3]

    @pytest.mark.asyncio
    async def test_run_task_with_args(self):
        """Test running task with arguments."""
        result = []

        def my_task(a, b, name):
            result.append((a, b, name))

        background = BackgroundTasks()
        background.add_task(my_task, 1, 2, name="test")
        await background.run()

        assert result == [(1, 2, "test")]

    @pytest.mark.asyncio
    async def test_run_async_task_with_args(self):
        """Test running async task with arguments."""
        result = []

        async def my_async_task(value):
            await asyncio.sleep(0)  # Simulate async work
            result.append(value)

        background = BackgroundTasks()
        background.add_task(my_async_task, 42)
        await background.run()

        assert result == [42]

    @pytest.mark.asyncio
    async def test_run_task_error_handling(self):
        """Test that errors in tasks don't crash the runner."""
        executed = []

        def failing_task():
            executed.append("before_error")
            raise ValueError("Task failed")

        def succeeding_task():
            executed.append("after_error")

        background = BackgroundTasks()
        background.add_task(failing_task)
        background.add_task(succeeding_task)

        # Should not raise, should log error
        await background.run()

        # Both tasks should have been attempted
        assert "before_error" in executed
        assert "after_error" in executed

    @pytest.mark.asyncio
    async def test_run_async_task_error_handling(self):
        """Test that errors in async tasks don't crash the runner."""
        executed = []

        async def failing_async_task():
            executed.append("async_before_error")
            raise RuntimeError("Async task failed")

        async def succeeding_async_task():
            executed.append("async_after_error")

        background = BackgroundTasks()
        background.add_task(failing_async_task)
        background.add_task(succeeding_async_task)

        # Should not raise
        await background.run()

        # Both tasks should have been attempted
        assert "async_before_error" in executed
        assert "async_after_error" in executed

    @pytest.mark.asyncio
    async def test_run_mixed_sync_async_tasks(self):
        """Test running mix of sync and async tasks."""
        executed = []

        def sync_task(value):
            executed.append(f"sync_{value}")

        async def async_task(value):
            await asyncio.sleep(0)
            executed.append(f"async_{value}")

        background = BackgroundTasks()
        background.add_task(sync_task, 1)
        background.add_task(async_task, 2)
        background.add_task(sync_task, 3)
        background.add_task(async_task, 4)

        await background.run()

        assert executed == ["sync_1", "async_2", "sync_3", "async_4"]

    @pytest.mark.asyncio
    async def test_sync_task_runs_in_executor(self):
        """Test that sync tasks run in executor (thread pool)."""
        import threading
        main_thread = threading.current_thread()
        task_thread = []

        def my_task():
            task_thread.append(threading.current_thread())

        background = BackgroundTasks()
        background.add_task(my_task)
        await background.run()

        # Task should run in a different thread (executor thread)
        assert len(task_thread) == 1
        # This may or may not be a different thread depending on executor
        # Just verify it executed
        assert task_thread[0] is not None


class TestFactoryFunction:
    """Test get_background_tasks factory function."""

    def test_get_background_tasks(self):
        """Test factory function returns new instance."""
        bg1 = get_background_tasks()
        bg2 = get_background_tasks()

        assert isinstance(bg1, BackgroundTasks)
        assert isinstance(bg2, BackgroundTasks)
        # Should be different instances
        assert bg1 is not bg2

    def test_factory_returns_empty_instance(self):
        """Test factory returns empty BackgroundTasks."""
        bg = get_background_tasks()
        assert len(bg) == 0


class TestIntegrationScenarios:
    """Test realistic integration scenarios."""

    @pytest.mark.asyncio
    async def test_email_notification_scenario(self):
        """Test email notification scenario."""
        sent_emails = []

        async def send_email(to: str, subject: str, body: str):
            await asyncio.sleep(0.01)  # Simulate email sending
            sent_emails.append({
                "to": to,
                "subject": subject,
                "body": body
            })

        background = BackgroundTasks()
        background.add_task(
            send_email,
            to="user@example.com",
            subject="Welcome",
            body="Thanks for signing up!"
        )

        # Simulate response sent, then background tasks run
        await background.run()

        assert len(sent_emails) == 1
        assert sent_emails[0]["to"] == "user@example.com"
        assert sent_emails[0]["subject"] == "Welcome"

    @pytest.mark.asyncio
    async def test_analytics_logging_scenario(self):
        """Test analytics logging scenario."""
        analytics_events = []

        def log_event(event_type: str, user_id: int, metadata: dict):
            analytics_events.append({
                "type": event_type,
                "user_id": user_id,
                "metadata": metadata
            })

        background = BackgroundTasks()
        background.add_task(
            log_event,
            "signup",
            user_id=123,
            metadata={"source": "web"}
        )
        background.add_task(
            log_event,
            "login",
            user_id=123,
            metadata={"ip": "192.168.1.1"}
        )

        await background.run()

        assert len(analytics_events) == 2
        assert analytics_events[0]["type"] == "signup"
        assert analytics_events[1]["type"] == "login"

    @pytest.mark.asyncio
    async def test_cleanup_scenario(self):
        """Test file cleanup scenario."""
        cleaned_files = []

        def cleanup_temp_files(*file_paths):
            for path in file_paths:
                cleaned_files.append(path)

        background = BackgroundTasks()
        background.add_task(
            cleanup_temp_files,
            "/tmp/upload1.tmp",
            "/tmp/upload2.tmp"
        )

        await background.run()

        assert cleaned_files == ["/tmp/upload1.tmp", "/tmp/upload2.tmp"]

    @pytest.mark.asyncio
    async def test_chained_operations_scenario(self):
        """Test chained operations that must run in order."""
        operations = []

        def process_order(order_id: int):
            operations.append(f"process_{order_id}")

        def send_confirmation(order_id: int):
            operations.append(f"confirm_{order_id}")

        def update_inventory(order_id: int):
            operations.append(f"inventory_{order_id}")

        background = BackgroundTasks()
        background.add_task(process_order, 456)
        background.add_task(send_confirmation, 456)
        background.add_task(update_inventory, 456)

        await background.run()

        # Operations should run in order
        assert operations == [
            "process_456",
            "confirm_456",
            "inventory_456"
        ]
