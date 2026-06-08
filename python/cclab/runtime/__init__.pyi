from typing import Any

__all__ = ["WakerCallback", "Handle", "TimerHandle", "PyLoop", "Task", "PyFuture"]

class WakerCallback:
    """Python-exposed wrapper for PythonWaker.

This allows Python code to trigger the waker when an awaited
operation completes."""
    def __call__(self, _result: Any | None) -> None:
        """Trigger the waker (called by Python when operation completes)."""
        ...
    def is_done(self) -> bool:
        """Check if already triggered."""
        ...

class Handle:
    """Handle to a scheduled callback

Represents a scheduled callback in the event loop. The callback can be
cancelled by calling `cancel()` before it executes.

# Thread Safety

Handle is thread-safe and can be passed between threads. The cancellation
flag uses atomic operations for lock-free cancellation.

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
handle = loop.call_soon(print, "Hello")
handle.cancel()  # Cancel before execution
```"""
    def cancelled(self) -> bool:
        """Check if the callback has been cancelled

Returns:
bool: True if the callback was cancelled, False otherwise"""
        ...
    def cancel(self) -> None:
        """Cancel the callback

If the callback has not yet been executed, it will be skipped.
Calling cancel() on an already-executed or already-cancelled handle
has no effect."""
        ...
    def __repr__(self) -> str:
        """Get debug representation"""
        ...

class TimerHandle:
    """Handle to a scheduled timer callback

Provides timer-specific functionality, including the ability
to cancel the underlying Tokio task that implements the delay.

# Thread Safety

TimerHandle is thread-safe and can be passed between threads.

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
handle = loop.call_later(1.0, print, "Hello after 1 second")
handle.cancel()  # Cancel before execution
```"""
    def cancel(self) -> None:
        """Cancel the timer

Cancels both the base handle and aborts the underlying Tokio task."""
        ...
    def cancelled(self) -> bool:
        """Check if cancelled"""
        ...
    def __repr__(self) -> str:
        """Get debug representation"""
        ...

class PyLoop:
    """PyLoop: Rust-native Python asyncio event loop backed by Tokio

This class implements the Python asyncio event loop protocol,
delegating all actual work to a Tokio runtime. This provides:

- High performance through Rust's async runtime
- Better integration with native Rust async code
- Reduced GIL contention through strategic GIL release
- Native support for spawning Rust futures from Python

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
# Use like any asyncio event loop
```"""
    def __init__(self) -> None:
        """Create a new PyLoop instance (Python constructor)

This creates a new event loop backed by the shared Tokio runtime."""
        ...
    def is_running(self) -> bool:
        """Check if the event loop is running"""
        ...
    def is_closed(self) -> bool:
        """Check if the event loop is closed"""
        ...
    def set_debug(self, enabled: bool) -> None:
        """Enable or disable debug mode

When debug mode is enabled:
- Slow callbacks are logged
- Resource warnings are enabled
- More verbose error messages are produced

# Arguments

* `enabled` - True to enable debug mode, False to disable"""
        ...
    def get_debug(self) -> bool:
        """Get the current debug mode state

Returns:
bool: True if debug mode is enabled, False otherwise"""
        ...
    def close(self) -> None:
        """Close the event loop

This marks the loop as closed but does not shut down the shared
Tokio runtime (which may be used by other PyLoop instances)."""
        ...
    def shutdown_with_timeout(self, timeout_secs: float = 30.0) -> bool:
        """Gracefully shutdown the event loop with a timeout

This initiates a graceful shutdown sequence:
1. Sets the stopped flag to prevent new callbacks
2. Drains the task queue, executing pending callbacks
3. Stops the timer wheel
4. Marks the loop as closed

If the timeout is reached before all tasks complete, remaining
tasks are dropped and the loop is forcefully closed.

# Arguments

* `timeout_secs` - Maximum time to wait for tasks to complete (in seconds)

# Returns

* `Ok(true)` - Shutdown completed gracefully (all tasks drained)
* `Ok(false)` - Shutdown completed with timeout (some tasks dropped)
* `Err` - Shutdown failed

# Example

```python
loop = PyLoop()
# ... schedule some tasks ...
graceful = loop.shutdown_with_timeout(5.0)  # Wait up to 5 seconds
if graceful:
print("All tasks completed")
else:
print("Timeout - some tasks were dropped")
```"""
        ...
    def call_soon(self, callback: Any, args: tuple[Any, ...], *args) -> Any:
        """Schedule a callback to be called soon

Arrange for `callback(*args)` to be called on the next iteration
of the event loop. Callbacks are called in the order in which they
are registered. Each callback will be called exactly once.

Any positional arguments after the callback will be passed to the
callback when it is called.

An instance of `Handle` is returned, which can be used to cancel
the callback.

This method is not thread-safe. Use `call_soon_threadsafe` to
schedule callbacks from other threads.

Args:
callback: The function to call
*args: Positional arguments to pass to the callback

Returns:
Handle: A handle that can be used to cancel the callback

Raises:
RuntimeError: If the event loop is closed

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
handle = loop.call_soon(print, "Hello, World!")
```"""
        ...
    def call_soon_threadsafe(self, callback: Any, args: tuple[Any, ...], *args) -> Any:
        """Schedule a callback to be called soon (thread-safe)

Like `call_soon`, but thread-safe. This method can be called from
any thread to schedule a callback in the event loop's thread.

Args:
callback: The function to call
*args: Positional arguments to pass to the callback

Returns:
Handle: A handle that can be used to cancel the callback

Raises:
RuntimeError: If the event loop is closed

# Example

```python
from cclab._pyloop import PyLoop
import threading

loop = PyLoop()

def worker():
loop.call_soon_threadsafe(print, "From thread!")

thread = threading.Thread(target=worker)
thread.start()
```"""
        ...
    def call_later(self, delay: float, callback: Any, args: tuple[Any, ...], *args) -> Any:
        """Schedule a callback to be called after a delay

Arrange for `callback(*args)` to be called approximately `delay` seconds
in the future. The delay is relative to the current time.

Args:
delay: Delay in seconds (float, must be non-negative)
callback: The function to call
*args: Positional arguments to pass to the callback

Returns:
TimerHandle: A handle that can be used to cancel the callback

Raises:
RuntimeError: If the event loop is closed
ValueError: If delay is negative

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
handle = loop.call_later(1.0, print, "Hello after 1 second")
```"""
        ...
    def call_at(self, when: float, callback: Any, args: tuple[Any, ...], *args) -> Any:
        """Schedule a callback to be called at an absolute time

Arrange for `callback(*args)` to be called at the given absolute
timestamp `when` (a float using the same time reference as `time()`).

Args:
when: Absolute time in seconds (float)
callback: The function to call
*args: Positional arguments to pass to the callback

Returns:
TimerHandle: A handle that can be used to cancel the callback

Raises:
RuntimeError: If the event loop is closed

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
when = loop.time() + 1.0  # 1 second from now
handle = loop.call_at(when, print, "Hello")
```"""
        ...
    def time(self) -> float:
        """Get the loop's internal time

Returns the current time according to the event loop's internal clock.
The time is a float representing seconds since an arbitrary reference point.

Returns:
float: The current loop time in seconds

# Example

```python
from cclab._pyloop import PyLoop

loop = PyLoop()
now = loop.time()
```"""
        ...
    def create_task(self, coro: Any, name: str | None = None) -> Any:
        """Create a task from a coroutine

Wrap a coroutine in a Task and schedule it for execution. The coroutine
will start executing on the next iteration of the event loop.

# Arguments

* `coro` - A Python coroutine object (must have a `send` method)
* `name` - Optional task name for debugging

# Returns

A Task object that wraps the coroutine

# Raises

* `RuntimeError` - If the event loop is closed
* `TypeError` - If the argument is not a coroutine

# Example

```python
async def my_coro():
await asyncio.sleep(1)
return 42

task = loop.create_task(my_coro())
result = await task  # Returns 42
```"""
        ...
    def stop(self) -> None:
        """Stop the event loop

This will cause `run_forever` to exit after the current iteration.

# Example

```python
loop = PyLoop()

def stop_soon():
loop.stop()

loop.call_later(1.0, stop_soon)
loop.run_forever()  # Will stop after 1 second
```"""
        ...
    def run_forever(self) -> None:
        """Run the event loop until stop() is called

Processes all scheduled callbacks (from call_soon, call_later, etc.)
in a continuous loop until stop() is called.

# Example

```python
loop = PyLoop()

def hello():
print("Hello from event loop!")
loop.stop()

loop.call_soon(hello)
loop.run_forever()  # Prints "Hello from event loop!" and exits
```"""
        ...
    def run_until_complete(self, future: Any) -> Any:
        """Run the event loop until a future completes

# Arguments

* `future` - A Task or coroutine to run until completion

# Returns

The result of the future

# Example

```python
loop = PyLoop()

async def my_coro():
await asyncio.sleep(1)
return 42

result = loop.run_until_complete(my_coro())
print(result)  # 42
```"""
        ...
    def __repr__(self) -> str:
        """Get debug representation"""
        ...

class Task:
    """Task wrapping a Python coroutine

A Task represents a scheduled coroutine that will run to completion.
It can be awaited, cancelled, or have its result retrieved.

# Thread Safety

Task is thread-safe and uses atomic flags for state management.

# Example

```python
async def my_coro():
await asyncio.sleep(1)
return 42

task = loop.create_task(my_coro())
result = await task  # Returns 42
```"""
    def cancel(self) -> bool:
        """Cancel the task

Request that the task be cancelled. Increments the cancellation depth.
If the task has already completed, this method has no effect.
Returns True if the cancellation request was recorded, False if already done.

Multiple calls to cancel() increment the depth; use uncancel() to decrement.
The task is considered cancelled when cancel_depth > 0.

Returns:
bool: True if cancellation recorded, False if already done

# Example

```python
task = loop.create_task(my_coro())
success = task.cancel()
assert task.cancelled()
task.uncancel()  # Decrement depth
```"""
        ...
    def uncancel(self) -> int:
        """Decrement the cancellation count

This method decrements the internal cancellation counter. It should be
called to undo a previous cancel() call. Returns the new cancellation depth.

Note: Once a task is done, uncancel() has no effect on the done state.

Returns:
int: The new cancellation depth (0 means not cancelled)

# Example

```python
task.cancel()  # depth = 1
task.cancel()  # depth = 2
depth = task.uncancel()  # depth = 1
```"""
        ...
    def cancelling(self) -> int:
        """Get the current cancellation count

Returns the number of pending cancel() calls that have not been
offset by uncancel() calls.

Returns:
int: The cancellation depth"""
        ...
    def cancelled(self) -> bool:
        """Check if the task has been cancelled

A task is considered cancelled when its cancellation depth > 0.

Returns:
bool: True if cancelled, False otherwise"""
        ...
    def done(self) -> bool:
        """Check if the task is done

A task is done when it has finished execution (successfully or with an
exception) or when it has been cancelled.

Returns:
bool: True if done, False otherwise"""
        ...
    def result(self) -> Any:
        """Get the task result

Return the result of the task. If the task is done, the result is
returned (or the exception is re-raised). If the task has been
cancelled, a CancelledError is raised. If the task is not done yet,
a RuntimeError is raised.

Returns:
object: The result of the coroutine

Raises:
RuntimeError: If task is not done yet
CancelledError: If task was cancelled
Exception: The exception raised by the coroutine

# Example

```python
task = loop.create_task(my_coro())
# ... wait for task to complete ...
result = task.result()  # Returns the coroutine's return value
```"""
        ...
    def get_name(self) -> str | None:
        """Get the task name

Returns:
Optional[str]: The task name if set, None otherwise"""
        ...
    def set_name(self, name: str) -> None:
        """Set the task name

Args:
name (str): The new task name"""
        ...
    def __repr__(self) -> str:
        """Get debug representation"""
        ...

class PyFuture:
    """PyFuture: A handle to a running future in the Tokio runtime

This represents a task that has been spawned on the event loop.
It can be awaited from Python code to get the result.

# Example

```python
# This will be implemented in later phases
future = loop.create_task(my_coroutine())
result = await future
```"""
    def __repr__(self) -> str:
        """Get debug representation"""
        ...

