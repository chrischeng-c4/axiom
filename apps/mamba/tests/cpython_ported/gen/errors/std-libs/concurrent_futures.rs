use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/broken_executor_is_runtimeerror.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_broken_executor_is_runtimeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "broken_executor_is_runtimeerror"
# subject = "concurrent.futures.BrokenExecutor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.BrokenExecutor: concurrent.futures.BrokenExecutor is a subclass of RuntimeError"""
import concurrent.futures

assert issubclass(concurrent.futures.BrokenExecutor, RuntimeError), "BrokenExecutor is a RuntimeError"
# A RuntimeError handler therefore catches a raised BrokenExecutor.
raised = False
try:
    raise concurrent.futures.BrokenExecutor("pool broke")
except RuntimeError as e:
    raised = isinstance(e, concurrent.futures.BrokenExecutor)
assert raised, "BrokenExecutor caught as RuntimeError"

print("broken_executor_is_runtimeerror OK")
"###);
    assert_output(&out, r###"broken_executor_is_runtimeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/cancel_after_done_returns_false.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_cancel_after_done_returns_false() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "cancel_after_done_returns_false"
# subject = "concurrent.futures.Future.cancel"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.cancel: calling Future.cancel() after a result has been set returns False and does NOT raise (the completed future is uncancellable)"""
import concurrent.futures

f = concurrent.futures.Future()
f.set_result(1)
assert f.done(), "future is done after set_result"
# A completed future cannot be cancelled: cancel() returns False, no raise.
assert f.cancel() is False, "cancel() on a done future returns False"
assert f.cancelled() is False, "a done future is not cancelled"
assert f.result() == 1, "result is still readable after the failed cancel"

print("cancel_after_done_returns_false OK")
"###);
    assert_output(&out, r###"cancel_after_done_returns_false OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/cancelled_error_is_exception.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_cancelled_error_is_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "cancelled_error_is_exception"
# subject = "concurrent.futures.CancelledError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.CancelledError: concurrent.futures.CancelledError is a subclass of BaseException (and of Exception in 3.12)"""
import concurrent.futures

assert issubclass(concurrent.futures.CancelledError, BaseException), "CancelledError is a BaseException"
assert issubclass(concurrent.futures.CancelledError, Exception), "CancelledError is an Exception in 3.12"
# It is also raise/catchable as a normal exception.
raised = False
try:
    raise concurrent.futures.CancelledError("cancelled")
except Exception as e:
    raised = isinstance(e, concurrent.futures.CancelledError)
assert raised, "CancelledError caught as Exception"

print("cancelled_error_is_exception OK")
"###);
    assert_output(&out, r###"cancelled_error_is_exception OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/result_after_set_exception_raises.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_result_after_set_exception_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "result_after_set_exception_raises"
# subject = "concurrent.futures.Future.result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: result_after_set_exception_raises (errors)."""
from concurrent.futures import Future
_f = Future()
_f.set_exception(ValueError('inner'))

_raised = False
try:
    _f.result()
except ValueError:
    _raised = True
assert _raised, "result_after_set_exception_raises: expected ValueError"
print("result_after_set_exception_raises OK")
"###);
    assert_output(&out, r###"result_after_set_exception_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/result_timeout_on_pending_raises.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_result_timeout_on_pending_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "result_timeout_on_pending_raises"
# subject = "concurrent.futures.Future.result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: Future.result(timeout) on a future whose task is still running raises concurrent.futures.TimeoutError before the task finishes"""
import concurrent.futures
import threading

# A task that blocks until released so the future stays pending across the
# short result() timeout window.
release = threading.Event()


def blocker():
    release.wait(10)
    return 1


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(blocker)
    raised = False
    try:
        fut.result(timeout=0.01)
    except concurrent.futures.TimeoutError:
        raised = True
    assert raised, "result(timeout) on a pending future must raise TimeoutError"
    # Release the worker so the executor drains cleanly on context exit.
    release.set()

assert fut.result(timeout=5) == 1, "task still completes once released"

print("result_timeout_on_pending_raises OK")
"###);
    assert_output(&out, r###"result_timeout_on_pending_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/set_result_twice_raises.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_set_result_twice_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "set_result_twice_raises"
# subject = "concurrent.futures.Future.set_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.set_result: set_result_twice_raises (errors)."""
from concurrent.futures import Future, InvalidStateError
_f = Future()
_f.set_result(1)

_raised = False
try:
    _f.set_result(2)
except InvalidStateError:
    _raised = True
assert _raised, "set_result_twice_raises: expected InvalidStateError"
print("set_result_twice_raises OK")
"###);
    assert_output(&out, r###"set_result_twice_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/concurrent_futures/submit_after_shutdown_raises.py`.
#[test]
fn test_gen_errors_std_libs_concurrent_futures_submit_after_shutdown_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "submit_after_shutdown_raises"
# subject = "concurrent.futures.Executor.submit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.submit: submit_after_shutdown_raises (errors)."""
from concurrent.futures import ThreadPoolExecutor
_ex = ThreadPoolExecutor(max_workers=1)
_ex.shutdown(wait=True)

_raised = False
try:
    _ex.submit(lambda: 1)
except RuntimeError:
    _raised = True
assert _raised, "submit_after_shutdown_raises: expected RuntimeError"
print("submit_after_shutdown_raises OK")
"###);
    assert_output(&out, r###"submit_after_shutdown_raises OK
"###);
}
