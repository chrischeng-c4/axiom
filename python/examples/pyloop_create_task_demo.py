#!/usr/bin/env python3
"""
Demo of PyLoop create_task functionality.

This demonstrates Phase 2.4 implementation of create_task method.
"""

import time
from cclab.orbit import PyLoop


def demo_basic_task():
    """Demo: Create and wait for a simple task."""
    print("\n=== Demo 1: Basic Task ===")
    loop = PyLoop()

    async def my_coro():
        """A simple coroutine that returns a value."""
        return 42

    task = loop.create_task(my_coro(), name="demo_task")
    print(f"Created: {task}")
    print(f"Task name: {task.get_name()}")
    print(f"Done initially: {task.done()}")

    # Wait for completion
    for _ in range(50):
        if task.done():
            break
        time.sleep(0.01)

    if task.done():
        print(f"Task completed!")
        print(f"Result: {task.result()}")
    else:
        print("Task still running...")


def demo_task_cancellation():
    """Demo: Cancel a task."""
    print("\n=== Demo 2: Task Cancellation ===")
    loop = PyLoop()

    async def long_running():
        """A long-running coroutine."""
        # This would normally take a long time
        pass

    task = loop.create_task(long_running(), name="long_task")
    print(f"Created: {task}")

    # Cancel it immediately
    cancelled = task.cancel()
    print(f"Cancelled: {cancelled}")
    print(f"Is cancelled: {task.cancelled()}")
    print(f"Is done: {task.done()}")

    # Try to get result (should raise CancelledError)
    try:
        task.result()
    except Exception as e:
        print(f"Expected error: {type(e).__name__}: {e}")


def demo_task_exception():
    """Demo: Handle exception in task."""
    print("\n=== Demo 3: Task Exception Handling ===")
    loop = PyLoop()

    async def failing_coro():
        """A coroutine that raises an exception."""
        raise ValueError("Something went wrong!")

    task = loop.create_task(failing_coro(), name="failing_task")
    print(f"Created: {task}")

    # Wait for completion
    for _ in range(50):
        if task.done():
            break
        time.sleep(0.01)

    if task.done():
        print("Task completed (with exception)")
        try:
            task.result()
        except ValueError as e:
            print(f"Caught exception: {e}")


def demo_multiple_tasks():
    """Demo: Create multiple tasks."""
    print("\n=== Demo 4: Multiple Tasks ===")
    loop = PyLoop()

    async def task_coro(task_id):
        """A coroutine identified by task_id."""
        return f"Task {task_id} result"

    # Create multiple tasks
    tasks = []
    for i in range(5):
        task = loop.create_task(task_coro(i), name=f"task_{i}")
        tasks.append(task)
        print(f"Created: {task}")

    # Wait for all to complete
    print("\nWaiting for tasks to complete...")
    max_wait = 100
    for _ in range(max_wait):
        if all(t.done() for t in tasks):
            break
        time.sleep(0.01)

    # Print results
    print("\nResults:")
    for task in tasks:
        if task.done() and not task.cancelled():
            try:
                result = task.result()
                print(f"  {task.get_name()}: {result}")
            except Exception as e:
                print(f"  {task.get_name()}: Error - {e}")


def demo_task_states():
    """Demo: Task state transitions."""
    print("\n=== Demo 5: Task States ===")
    loop = PyLoop()

    async def simple_coro():
        return "done"

    task = loop.create_task(simple_coro(), name="state_task")

    # Initial state
    print("Initial state:")
    print(f"  done(): {task.done()}")
    print(f"  cancelled(): {task.cancelled()}")
    print(f"  repr: {repr(task)}")

    # Wait for completion
    time.sleep(0.1)

    # Final state
    if task.done():
        print("\nFinal state:")
        print(f"  done(): {task.done()}")
        print(f"  cancelled(): {task.cancelled()}")
        print(f"  repr: {repr(task)}")
        print(f"  result: {task.result()}")


if __name__ == "__main__":
    print("=" * 60)
    print("PyLoop create_task Demo (Phase 2.4)")
    print("=" * 60)

    demo_basic_task()
    demo_task_cancellation()
    demo_task_exception()
    demo_multiple_tasks()
    demo_task_states()

    print("\n" + "=" * 60)
    print("Demo completed!")
    print("=" * 60)
