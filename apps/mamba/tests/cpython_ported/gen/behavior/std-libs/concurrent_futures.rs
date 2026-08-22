use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/add_done_callback_fires.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_add_done_callback_fires() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "add_done_callback_fires"
# subject = "concurrent.futures.Future.add_done_callback"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.add_done_callback: a callback registered with Future.add_done_callback is invoked once the future completes and observes the future's result"""
import concurrent.futures
import threading

fired = threading.Event()
seen = []


def on_done(fut):
    seen.append(fut.result())
    fired.set()


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: "callback_value")
    fut.add_done_callback(on_done)

# After the executor drains, the callback has run (event-based, no polling race).
assert fired.wait(5), "add_done_callback fired within budget"
assert seen == ["callback_value"], f"callback observed the result: {seen!r}"

print("add_done_callback_fires OK")
"###);
    assert_output(&out, r###"add_done_callback_fires OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/as_completed_yields_all_futures.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_as_completed_yields_all_futures() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "as_completed_yields_all_futures"
# subject = "concurrent.futures.as_completed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.as_completed: as_completed yields every submitted future exactly once as it finishes; collecting results over range(5) squares gives the full set {0,1,4,9,16}"""
import concurrent.futures


def square(n):
    return n * n


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(square, i) for i in range(5)]
    done_results = []
    for f in concurrent.futures.as_completed(futs, timeout=5):
        done_results.append(f.result())

assert len(done_results) == 5, f"as_completed yielded {len(done_results)} futures, expected 5"
assert sorted(done_results) == [0, 1, 4, 9, 16], f"as_completed results = {sorted(done_results)!r}"

print("as_completed_yields_all_futures OK")
"###);
    assert_output(&out, r###"as_completed_yields_all_futures OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/context_manager_completes_futures.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_context_manager_completes_futures() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "context_manager_completes_futures"
# subject = "concurrent.futures.ThreadPoolExecutor.__exit__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.__exit__: using the executor as a context manager blocks on __exit__ until all in-flight futures are done(), so every future reports done() True after the with-block"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    futs = [ex.submit(lambda x=i: x, i) for i in range(4)]

# __exit__ shut the pool down with wait=True, so every future is settled.
for f in futs:
    assert f.done() is True, "future done after executor context exit"
assert sorted(f.result() for f in futs) == [0, 1, 2, 3], "all futures carry their results"

print("context_manager_completes_futures OK")
"###);
    assert_output(&out, r###"context_manager_completes_futures OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/exception_captures_task_error.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_exception_captures_task_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "exception_captures_task_error"
# subject = "concurrent.futures.Future.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.exception: Future.exception() returns the exception object raised inside the task (a ValueError with its original message) rather than re-raising it"""
import concurrent.futures


def raises():
    raise ValueError("test error")


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(raises)
    exc = fut.exception(timeout=5)
    assert isinstance(exc, ValueError), f"captured exception type = {type(exc)!r}"
    assert str(exc) == "test error", f"captured exception msg = {str(exc)!r}"

print("exception_captures_task_error OK")
"###);
    assert_output(&out, r###"exception_captures_task_error OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/exception_none_for_success.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_exception_none_for_success() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "exception_none_for_success"
# subject = "concurrent.futures.Future.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.exception: Future.exception() returns None for a future that completed successfully"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: 99)
    assert fut.result(timeout=5) == 99, "task succeeds"
    assert fut.exception(timeout=5) is None, "exception() is None for a successful future"

print("exception_none_for_success OK")
"###);
    assert_output(&out, r###"exception_none_for_success OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/future_state_done_not_cancelled.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_future_state_done_not_cancelled() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "future_state_done_not_cancelled"
# subject = "concurrent.futures.Future.done"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.done: after .result() returns, a successful future reports done() True and cancelled() False"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: "hello")
    val = fut.result(timeout=5)
    assert val == "hello", f"future value = {val!r}"
    assert fut.done() is True, "future is done after result"
    assert fut.cancelled() is False, "successful future is not cancelled"

print("future_state_done_not_cancelled OK")
"###);
    assert_output(&out, r###"future_state_done_not_cancelled OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/map_applies_over_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_map_applies_over_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "map_applies_over_iterable"
# subject = "concurrent.futures.Executor.map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.map: Executor.map applies the function to each element and yields results in input order: map(x*2, [1,2,3,4]) -> [2,4,6,8]"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    results = list(ex.map(lambda x: x * 2, [1, 2, 3, 4], timeout=5))
    assert results == [2, 4, 6, 8], f"map results (in input order) = {results!r}"

print("map_applies_over_iterable OK")
"###);
    assert_output(&out, r###"map_applies_over_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/map_propagates_task_exception.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_map_propagates_task_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "map_propagates_task_exception"
# subject = "concurrent.futures.Executor.map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.map: if any mapped task raises, iterating the Executor.map result propagates that exception (ValueError) to the consumer"""
import concurrent.futures


def raise_on_2(x):
    if x == 2:
        raise ValueError(f"bad: {x}")
    return x


with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    raised = False
    try:
        list(ex.map(raise_on_2, [0, 1, 2, 3], timeout=5))
    except ValueError:
        raised = True
assert raised, "Executor.map propagates the task's ValueError to the consumer"

print("map_propagates_task_exception OK")
"###);
    assert_output(&out, r###"map_propagates_task_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/max_workers_caps_concurrency.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_max_workers_caps_concurrency() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "max_workers_caps_concurrency"
# subject = "concurrent.futures.ThreadPoolExecutor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor: max_workers=2 caps simultaneously-running tasks at 2; a concurrency tracker across six tasks never observes more than two active at once"""
import concurrent.futures
import threading
import time

active = [0]
max_active = [0]
lock = threading.Lock()


def track_concurrency():
    with lock:
        active[0] += 1
        if active[0] > max_active[0]:
            max_active[0] = active[0]
    time.sleep(0.05)
    with lock:
        active[0] -= 1


with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    futs = [ex.submit(track_concurrency) for _ in range(6)]
    for f in futs:
        f.result(timeout=5)

assert max_active[0] <= 2, f"max_workers=2 cap respected: peak active = {max_active[0]!r}"
assert max_active[0] >= 1, "at least one task observed running"

print("max_workers_caps_concurrency OK")
"###);
    assert_output(&out, r###"max_workers_caps_concurrency OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/nested_submit_resolves.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_nested_submit_resolves() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "nested_submit_resolves"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: a task may submit further tasks to the same pool and await their results; outer tasks i for range(3) returning inner i*10 give {0,10,20}"""
import concurrent.futures


def outer_task(n, executor):
    inner = executor.submit(lambda x=n: x * 10)
    return inner.result(timeout=5)


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    outer = [ex.submit(outer_task, i, ex) for i in range(3)]
    results = [f.result(timeout=5) for f in outer]

assert sorted(results) == [0, 10, 20], f"nested submit results = {sorted(results)!r}"

print("nested_submit_resolves OK")
"###);
    assert_output(&out, r###"nested_submit_resolves OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/result_reraises_task_exception.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_result_reraises_task_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "result_reraises_task_exception"
# subject = "concurrent.futures.Future.result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: an exception raised inside a task is re-raised (same type and message) when the future's .result() is called"""
import concurrent.futures


def bad():
    raise RuntimeError("task failed")


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(bad)

raised = False
try:
    fut.result(timeout=5)
except RuntimeError as e:
    raised = True
    assert str(e) == "task failed", f"re-raised message = {str(e)!r}"
assert raised, "RuntimeError re-raised from future.result()"

print("result_reraises_task_exception OK")
"###);
    assert_output(&out, r###"result_reraises_task_exception OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/submit_returns_future_with_result.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_submit_returns_future_with_result() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "submit_returns_future_with_result"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: ThreadPoolExecutor.submit returns a concurrent.futures.Future whose .result() yields the task's return value (submit(lambda: 42).result() == 42)"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    fut = ex.submit(lambda: 42)
    assert isinstance(fut, concurrent.futures.Future), f"submit returns a Future, got {type(fut)!r}"
    result = fut.result(timeout=5)
    assert result == 42, f"future result = {result!r}"

print("submit_returns_future_with_result OK")
"###);
    assert_output(&out, r###"submit_returns_future_with_result OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/thread_pool_runs_all_tasks.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_thread_pool_runs_all_tasks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "thread_pool_runs_all_tasks"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: every submitted task runs to completion under a 4-worker pool: submitting i->i*i for range(10) records all ten inputs and returns all ten squares"""
import concurrent.futures
import threading

seen = []
lock = threading.Lock()


def work(n):
    with lock:
        seen.append(n)
    return n * n


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(work, i) for i in range(10)]
    squares = [f.result(timeout=5) for f in futs]

assert sorted(seen) == list(range(10)), f"all ten tasks ran: {sorted(seen)!r}"
assert sorted(squares) == [i * i for i in range(10)], f"squares: {sorted(squares)!r}"

print("thread_pool_runs_all_tasks OK")
"###);
    assert_output(&out, r###"thread_pool_runs_all_tasks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/concurrent_futures/wait_partitions_done_not_done.py`.
#[test]
fn test_gen_behavior_std_libs_concurrent_futures_wait_partitions_done_not_done() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "wait_partitions_done_not_done"
# subject = "concurrent.futures.wait"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.wait: wait(..., return_when=ALL_COMPLETED) returns a (done, not_done) pair; with all five futures finished done has 5 and not_done is empty"""
import concurrent.futures


def identity(x):
    return x


with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:
    futs = [ex.submit(identity, i) for i in range(5)]
    done, not_done = concurrent.futures.wait(
        futs, timeout=5, return_when=concurrent.futures.ALL_COMPLETED
    )
    assert len(done) == 5, f"all five futures done: {len(done)!r}"
    assert len(not_done) == 0, f"none pending: {len(not_done)!r}"

print("wait_partitions_done_not_done OK")
"###);
    assert_output(&out, r###"wait_partitions_done_not_done OK
"###);
}
