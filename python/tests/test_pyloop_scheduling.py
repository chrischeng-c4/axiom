"""
Tests for PyLoop call_soon and call_soon_threadsafe functionality.

This test suite verifies Phase 2.2 of the data-bridge-pyloop implementation:
- call_soon: Schedule callbacks for execution
- call_soon_threadsafe: Thread-safe callback scheduling
- Handle: Cancellation support for scheduled callbacks
"""

import threading
import pytest
import cclab

# Access _pyloop as an attribute (it's a Rust submodule, not a Python module)
pyloop_module = cclab._pyloop
PyLoop = pyloop_module.PyLoop
Handle = pyloop_module.Handle


def test_pyloop_import():
    """Test that PyLoop and Handle can be imported."""
    loop = PyLoop()
    assert not loop.is_running()
    assert not loop.is_closed()


def test_call_soon_basic():
    """Test basic call_soon functionality."""
    loop = PyLoop()
    result = []

    # Schedule a callback
    handle = loop.call_soon(result.append, 42)

    # Verify handle is not cancelled
    assert not handle.cancelled()

    # Note: The callback won't execute yet (run_forever not implemented)
    # We're just testing that scheduling works without errors


def test_call_soon_with_multiple_args():
    """Test call_soon with multiple arguments."""
    loop = PyLoop()
    result = []

    def callback(a, b, c):
        result.append(a + b + c)

    # Schedule with multiple args
    handle = loop.call_soon(callback, 1, 2, 3)
    assert not handle.cancelled()


def test_call_soon_no_args():
    """Test call_soon with no arguments."""
    loop = PyLoop()
    result = []

    # Schedule callback with no args
    handle = loop.call_soon(result.append, "test")
    assert not handle.cancelled()


def test_handle_cancel():
    """Test Handle cancellation."""
    loop = PyLoop()
    result = []

    handle = loop.call_soon(result.append, "should_cancel")

    # Initially not cancelled
    assert not handle.cancelled()

    # Cancel it
    handle.cancel()

    # Now it should be cancelled
    assert handle.cancelled()


def test_handle_double_cancel():
    """Test that calling cancel() multiple times is safe."""
    loop = PyLoop()

    handle = loop.call_soon(lambda: None)

    # Cancel twice - should not raise
    handle.cancel()
    handle.cancel()

    assert handle.cancelled()


def test_call_soon_on_closed_loop():
    """Test that call_soon raises error on closed loop."""
    loop = PyLoop()
    loop.close()

    with pytest.raises(RuntimeError, match="closed"):
        loop.call_soon(lambda: None)


def test_call_soon_threadsafe_basic():
    """Test call_soon_threadsafe functionality."""
    loop = PyLoop()
    result = []

    # Schedule using call_soon_threadsafe
    handle = loop.call_soon_threadsafe(result.append, "thread_safe")

    assert not handle.cancelled()


def test_call_soon_threadsafe_from_thread():
    """Test call_soon_threadsafe can be called from another thread."""
    loop = PyLoop()
    handles = []
    errors = []

    def worker():
        try:
            # This should work from another thread
            handle = loop.call_soon_threadsafe(lambda x: x, 42)
            handles.append(handle)
        except Exception as e:
            errors.append(e)

    thread = threading.Thread(target=worker)
    thread.start()
    thread.join()

    # Should have scheduled without error
    assert len(errors) == 0
    assert len(handles) == 1
    assert not handles[0].cancelled()


def test_multiple_calls_scheduled():
    """Test scheduling multiple callbacks."""
    loop = PyLoop()
    handles = []

    # Schedule 10 callbacks
    for i in range(10):
        handle = loop.call_soon(lambda x: x * 2, i)
        handles.append(handle)

    # All handles should be valid and not cancelled
    assert len(handles) == 10
    for handle in handles:
        assert not handle.cancelled()


def test_cancel_some_handles():
    """Test cancelling some handles while leaving others active."""
    loop = PyLoop()
    handles = []

    # Schedule 5 callbacks
    for i in range(5):
        handle = loop.call_soon(lambda x: x, i)
        handles.append(handle)

    # Cancel every other one
    for i in range(0, 5, 2):
        handles[i].cancel()

    # Verify correct cancellation state
    assert handles[0].cancelled()
    assert not handles[1].cancelled()
    assert handles[2].cancelled()
    assert not handles[3].cancelled()
    assert handles[4].cancelled()


def test_handle_repr():
    """Test Handle __repr__ method."""
    loop = PyLoop()
    handle = loop.call_soon(lambda: None)

    # Check repr shows not cancelled
    repr_str = repr(handle)
    assert "Handle" in repr_str
    assert "false" in repr_str.lower() or "False" in repr_str

    # Cancel and check again
    handle.cancel()
    repr_str = repr(handle)
    assert "true" in repr_str.lower() or "True" in repr_str


def test_pyloop_repr():
    """Test PyLoop __repr__ method."""
    loop = PyLoop()
    repr_str = repr(loop)

    assert "PyLoop" in repr_str
    assert "running=False" in repr_str or "running=false" in repr_str
    assert "closed=False" in repr_str or "closed=false" in repr_str


def test_lambda_callbacks():
    """Test that lambda functions work as callbacks."""
    loop = PyLoop()

    # Various lambda forms
    handle1 = loop.call_soon(lambda: None)
    handle2 = loop.call_soon(lambda x: x * 2, 21)
    handle3 = loop.call_soon(lambda x, y: x + y, 10, 20)

    assert not handle1.cancelled()
    assert not handle2.cancelled()
    assert not handle3.cancelled()


def test_method_callbacks():
    """Test that method callbacks work."""
    class Counter:
        def __init__(self):
            self.value = 0

        def increment(self):
            self.value += 1

    loop = PyLoop()
    counter = Counter()

    handle = loop.call_soon(counter.increment)
    assert not handle.cancelled()


def test_call_soon_threadsafe_on_closed_loop():
    """Test that call_soon_threadsafe raises error on closed loop."""
    loop = PyLoop()
    loop.close()

    with pytest.raises(RuntimeError, match="closed"):
        loop.call_soon_threadsafe(lambda: None)


if __name__ == "__main__":
    # Run tests with pytest
    pytest.main([__file__, "-v"])
