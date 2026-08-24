use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/_asyncio/future_cancel_accepts_object_message.py`.
#[test]
fn test_gen_behavior_std_libs__asyncio_future_cancel_accepts_object_message() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "future_cancel_accepts_object_message"
# subject = "_asyncio.Future.cancel(msg)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
"""_asyncio.Future.cancel accepts an arbitrary object message."""

from _asyncio import Future


class _W:
    pass


fut = Future()
assert fut.cancel(_W()) is True
assert fut.cancelled() is True
print("future_cancel_accepts_object_message OK")
"###);
    assert_output(&out, r###"future_cancel_accepts_object_message OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/_asyncio/future_set_result_accepts_object.py`.
#[test]
fn test_gen_behavior_std_libs__asyncio_future_set_result_accepts_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "future_set_result_accepts_object"
# subject = "_asyncio.Future.set_result(result)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
"""_asyncio.Future.set_result accepts and returns an arbitrary result object."""

from _asyncio import Future


class _W:
    pass


token = _W()
fut = Future()
assert fut.set_result(token) is None
assert fut.done() is True
assert fut.result() is token
print("future_set_result_accepts_object OK")
"###);
    assert_output(&out, r###"future_set_result_accepts_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/_asyncio/private_future_awaited_helpers_absent.py`.
#[test]
fn test_gen_behavior_std_libs__asyncio_private_future_awaited_helpers_absent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "private_future_awaited_helpers_absent"
# subject = "_asyncio private future awaited helpers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/asyncio"
# status = "filled"
# ///
"""CPython 3.12 does not expose private future awaited helper functions from _asyncio."""

import _asyncio


assert not hasattr(_asyncio, "future_add_to_awaited_by")
assert not hasattr(_asyncio, "future_discard_from_awaited_by")
print("private_future_awaited_helpers_absent OK")
"###);
    assert_output(&out, r###"private_future_awaited_helpers_absent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/event_wakes_waiter_after_set.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_event_wakes_waiter_after_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "event_wakes_waiter_after_set"
# subject = "asyncio.Event"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Event not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Event: an asyncio.Event wakes a waiting coroutine only after another coroutine calls set(); the waiter observes 'setting' before 'woke'"""
import asyncio


async def _main():
    _ev = asyncio.Event()
    _log = []

    async def _waiter():
        _log.append("waiting")
        await _ev.wait()
        _log.append("woke")

    async def _setter():
        await asyncio.sleep(0)
        _log.append("setting")
        _ev.set()

    await asyncio.gather(_waiter(), _setter())
    assert "waiting" in _log, "waiter started"
    assert "setting" in _log, "setter ran"
    assert "woke" in _log, "waiter woke"
    assert _log.index("setting") < _log.index("woke"), "setter before woke"


asyncio.run(_main())

print("event_wakes_waiter_after_set OK")
"###);
    assert_output(&out, r###"event_wakes_waiter_after_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/future_cancelled_before_await_raises.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_future_cancelled_before_await_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "future_cancelled_before_await_raises"
# subject = "asyncio.Future"
# kind = "semantic"
# xfail = "mamba asyncio shim: get_running_loop / loop.create_future not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Future: awaiting a Future that was cancelled before completion raises CancelledError"""
import asyncio


async def _main():
    _loop = asyncio.get_running_loop()
    _fut = _loop.create_future()
    _fut.cancel()
    _raised = False
    try:
        await _fut
    except asyncio.CancelledError:
        _raised = True
    assert _raised, "awaiting a cancelled future raises CancelledError"


asyncio.run(_main())

print("future_cancelled_before_await_raises OK")
"###);
    assert_output(&out, r###"future_cancelled_before_await_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/future_result_when_not_done_raises.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_future_result_when_not_done_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "future_result_when_not_done_raises"
# subject = "asyncio.Future"
# kind = "semantic"
# xfail = "mamba asyncio shim: get_running_loop / loop.create_future not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Future: calling result() on a Future that is not done raises asyncio.InvalidStateError"""
import asyncio


async def _main():
    _loop = asyncio.get_running_loop()
    _fut = _loop.create_future()
    _raised = False
    try:
        _fut.result()
    except asyncio.InvalidStateError:
        _raised = True
    assert _raised, "result() on a not-done future raises InvalidStateError"
    _fut.cancel()  # cleanup so the loop doesn't warn on a pending future


asyncio.run(_main())

print("future_result_when_not_done_raises OK")
"###);
    assert_output(&out, r###"future_result_when_not_done_raises OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/gather_returns_results_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_gather_returns_results_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "gather_returns_results_in_order"
# subject = "asyncio.gather"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.gather: asyncio.gather runs coroutines concurrently and returns their results in submission order ([10, 20])"""
import asyncio


async def _main():
    _order = []

    async def _a():
        _order.append("a_start")
        await asyncio.sleep(0)
        _order.append("a_end")
        return 10

    async def _b():
        _order.append("b_start")
        await asyncio.sleep(0)
        _order.append("b_end")
        return 20

    _results = await asyncio.gather(_a(), _b())
    assert _results == [10, 20], f"gather order = {_results!r}"


asyncio.run(_main())

print("gather_returns_results_in_order OK")
"###);
    assert_output(&out, r###"gather_returns_results_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/lock_serializes_critical_section.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_lock_serializes_critical_section() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "lock_serializes_critical_section"
# subject = "asyncio.Lock"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Lock not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Lock: an asyncio.Lock used as an async context manager guarantees only one coroutine is inside the critical section at a time"""
import asyncio


async def _main():
    _lock = asyncio.Lock()
    _inside = []

    async def _worker(n):
        async with _lock:
            _inside.append(n)
            await asyncio.sleep(0)
            assert len(_inside) == 1, f"only one inside: {_inside!r}"
            _inside.pop()

    await asyncio.gather(_worker(1), _worker(2), _worker(3))


asyncio.run(_main())

print("lock_serializes_critical_section OK")
"###);
    assert_output(&out, r###"lock_serializes_critical_section OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/queue_preserves_fifo_order.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_queue_preserves_fifo_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "queue_preserves_fifo_order"
# subject = "asyncio.Queue"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Queue not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Queue: asyncio.Queue dequeues items in the same order they were enqueued (FIFO: [0,1,2,3,4])"""
import asyncio


async def _main():
    _q = asyncio.Queue()
    for _i in range(5):
        await _q.put(_i)
    _results = []
    while not _q.empty():
        _results.append(await _q.get())
    assert _results == [0, 1, 2, 3, 4], f"FIFO queue: {_results!r}"


asyncio.run(_main())

print("queue_preserves_fifo_order OK")
"###);
    assert_output(&out, r###"queue_preserves_fifo_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/run_executes_coroutine.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_run_executes_coroutine() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "run_executes_coroutine"
# subject = "asyncio.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.run: asyncio.run drives a coroutine to completion and returns its value (run(coro_returning_42) == 42)"""
import asyncio


async def _simple():
    return 42


_result = asyncio.run(_simple())
assert _result == 42, f"run result = {_result!r}"

print("run_executes_coroutine OK")
"###);
    assert_output(&out, r###"run_executes_coroutine OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/semaphore_caps_concurrency.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_semaphore_caps_concurrency() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "semaphore_caps_concurrency"
# subject = "asyncio.Semaphore"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Semaphore not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Semaphore: an asyncio.Semaphore(2) limits the number of coroutines in its critical section to at most 2 at once"""
import asyncio


async def _main():
    _sem = asyncio.Semaphore(2)
    _active = [0]
    _max_active = [0]

    async def _worker():
        async with _sem:
            _active[0] += 1
            if _active[0] > _max_active[0]:
                _max_active[0] = _active[0]
            await asyncio.sleep(0)
            _active[0] -= 1

    await asyncio.gather(*[_worker() for _ in range(5)])
    assert _max_active[0] <= 2, f"max_concurrent={_max_active[0]}"


asyncio.run(_main())

print("semaphore_caps_concurrency OK")
"###);
    assert_output(&out, r###"semaphore_caps_concurrency OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/task_cancellation_raises_cancelled_error.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_task_cancellation_raises_cancelled_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "task_cancellation_raises_cancelled_error"
# subject = "asyncio.create_task"
# kind = "semantic"
# xfail = "mamba asyncio shim: create_task returns a non-Task (int) lacking .cancel() (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.create_task: cancelling a running Task makes awaiting it raise CancelledError and sets task.cancelled() True"""
import asyncio


async def _main():
    async def _long():
        await asyncio.sleep(100)

    _t = asyncio.create_task(_long())
    await asyncio.sleep(0)  # let the task start
    _t.cancel()
    _raised = False
    try:
        await _t
    except asyncio.CancelledError:
        _raised = True
    assert _raised, "cancelled task raises CancelledError"
    assert _t.cancelled(), "task.cancelled() True"


asyncio.run(_main())

print("task_cancellation_raises_cancelled_error OK")
"###);
    assert_output(&out, r###"task_cancellation_raises_cancelled_error OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/task_exception_propagates_on_await.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_task_exception_propagates_on_await() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "task_exception_propagates_on_await"
# subject = "asyncio.create_task"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.create_task: awaiting a Task whose coroutine raised re-raises the original exception, and the task is done() and not cancelled()"""
import asyncio


async def _main():
    async def _raiser():
        raise ValueError("task error")

    _t = asyncio.create_task(_raiser())
    _raised = False
    try:
        await _t
    except ValueError as _e:
        assert str(_e) == "task error", f"exception msg = {str(_e)!r}"
        _raised = True
    assert _raised, "task exception propagated"
    assert _t.done() and not _t.cancelled(), "task done not cancelled"


asyncio.run(_main())

print("task_exception_propagates_on_await OK")
"###);
    assert_output(&out, r###"task_exception_propagates_on_await OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/asyncio/wait_for_timeout_raises_timeout_error.py`.
#[test]
fn test_gen_behavior_std_libs_asyncio_wait_for_timeout_raises_timeout_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "wait_for_timeout_raises_timeout_error"
# subject = "asyncio.wait_for"
# kind = "semantic"
# xfail = "mamba asyncio shim: wait_for does not enforce the timeout / raise TimeoutError (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.wait_for: asyncio.wait_for raises asyncio.TimeoutError when the wrapped coroutine exceeds the timeout"""
import asyncio


async def _main():
    async def _slow():
        await asyncio.sleep(100)

    _raised = False
    try:
        await asyncio.wait_for(_slow(), timeout=0.001)
    except asyncio.TimeoutError:
        _raised = True
    assert _raised, "wait_for timeout raises TimeoutError"


asyncio.run(_main())

print("wait_for_timeout_raises_timeout_error OK")
"###);
    assert_output(&out, r###"wait_for_timeout_raises_timeout_error OK
"###);
}
