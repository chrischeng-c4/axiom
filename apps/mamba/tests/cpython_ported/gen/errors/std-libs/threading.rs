use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/threading/bounded_semaphore_over_release_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_bounded_semaphore_over_release_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "bounded_semaphore_over_release_raises"
# subject = "threading.BoundedSemaphore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.BoundedSemaphore: BoundedSemaphore(2): acquire then release returns to the bound; one more release past the bound raises ValueError"""
import threading

sem = threading.BoundedSemaphore(2)
sem.acquire()
sem.release()  # back at the bound of 2
_raised = False
try:
    sem.release()  # past the bound
except ValueError:
    _raised = True
assert _raised, "expected ValueError on over-release"

print("bounded_semaphore_over_release_raises OK")
"###);
    assert_output(&out, r###"bounded_semaphore_over_release_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/condition_notify_without_lock_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_condition_notify_without_lock_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "condition_notify_without_lock_raises"
# subject = "threading.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Condition: condition_notify_without_lock_raises (errors)."""
import threading

_raised = False
try:
    threading.Condition().notify()
except RuntimeError:
    _raised = True
assert _raised, "condition_notify_without_lock_raises: expected RuntimeError"
print("condition_notify_without_lock_raises OK")
"###);
    assert_output(&out, r###"condition_notify_without_lock_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/condition_wait_without_lock_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_condition_wait_without_lock_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "condition_wait_without_lock_raises"
# subject = "threading.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Condition: condition_wait_without_lock_raises (errors)."""
import threading

_raised = False
try:
    threading.Condition().wait(timeout=0.001)
except RuntimeError:
    _raised = True
assert _raised, "condition_wait_without_lock_raises: expected RuntimeError"
print("condition_wait_without_lock_raises OK")
"###);
    assert_output(&out, r###"condition_wait_without_lock_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/daemonize_running_thread_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_daemonize_running_thread_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "daemonize_running_thread_raises"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: setting daemon=True on an already-running thread raises RuntimeError; gate the worker with an Event so the run is deterministic and joined"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

running = threading.Thread(target=hold)
running.start()
_raised = False
try:
    running.daemon = True
except RuntimeError:
    _raised = True
finally:
    gate.set()
    running.join()
assert _raised, "expected RuntimeError when daemonizing a running thread"

print("daemonize_running_thread_raises OK")
"###);
    assert_output(&out, r###"daemonize_running_thread_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/join_self_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_join_self_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "join_self_raises"
# subject = "threading.current_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.current_thread: join_self_raises (errors)."""
import threading

_raised = False
try:
    threading.current_thread().join()
except RuntimeError:
    _raised = True
assert _raised, "join_self_raises: expected RuntimeError"
print("join_self_raises OK")
"###);
    assert_output(&out, r###"join_self_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/join_unstarted_thread_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_join_unstarted_thread_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "join_unstarted_thread_raises"
# subject = "threading.Thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: join_unstarted_thread_raises (errors)."""
import threading

_raised = False
try:
    threading.Thread().join()
except RuntimeError:
    _raised = True
assert _raised, "join_unstarted_thread_raises: expected RuntimeError"
print("join_unstarted_thread_raises OK")
"###);
    assert_output(&out, r###"join_unstarted_thread_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/release_unheld_rlock_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_release_unheld_rlock_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "release_unheld_rlock_raises"
# subject = "threading.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.RLock: release_unheld_rlock_raises (errors)."""
import threading

_raised = False
try:
    threading.RLock().release()
except RuntimeError:
    _raised = True
assert _raised, "release_unheld_rlock_raises: expected RuntimeError"
print("release_unheld_rlock_raises OK")
"###);
    assert_output(&out, r###"release_unheld_rlock_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/release_unlocked_lock_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_release_unlocked_lock_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "release_unlocked_lock_raises"
# subject = "threading.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock: release_unlocked_lock_raises (errors)."""
import threading

_raised = False
try:
    threading.Lock().release()
except RuntimeError:
    _raised = True
assert _raised, "release_unlocked_lock_raises: expected RuntimeError"
print("release_unlocked_lock_raises OK")
"###);
    assert_output(&out, r###"release_unlocked_lock_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/threading/restart_started_thread_raises.py`.
#[test]
fn test_gen_errors_std_libs_threading_restart_started_thread_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "errors"
# case = "restart_started_thread_raises"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: starting an already-started-and-joined thread a second time raises RuntimeError ('threads can only be started once')"""
import threading

def noop():
    pass

t = threading.Thread(target=noop)
t.start()
t.join()
_raised = False
try:
    t.start()
except RuntimeError as e:
    _raised = True
    assert "once" in str(e), f"message = {str(e)!r}"
assert _raised, "expected RuntimeError on restart"

print("restart_started_thread_raises OK")
"###);
    assert_output(&out, r###"restart_started_thread_raises OK
"###);
}
