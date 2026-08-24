use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/threading/active_count_includes_main.py`.
#[test]
fn test_gen_behavior_std_libs_threading_active_count_includes_main() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "active_count_includes_main"
# subject = "threading.active_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.active_count: active_count() always includes the main thread, so it is >= 1"""
import threading

_cnt = threading.active_count()
assert isinstance(_cnt, int), f"active_count type = {type(_cnt)!r}"
assert _cnt >= 1, f"active_count = {_cnt!r}"

print("active_count_includes_main OK")
"###);
    assert_output(&out, r###"active_count_includes_main OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/auto_thread_name.py`.
#[test]
fn test_gen_behavior_std_libs_threading_auto_thread_name() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "auto_thread_name"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: an unnamed Thread gets an auto-generated 'Thread-N' name"""
import threading

auto = threading.Thread()
assert auto.name.startswith("Thread-"), f"auto name = {auto.name!r}"

print("auto_thread_name OK")
"###);
    assert_output(&out, r###"auto_thread_name OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/daemon_flag_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_threading_daemon_flag_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "daemon_flag_roundtrip"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: the Thread(daemon=...) constructor flag is observable via the .daemon attribute for both True and False"""
import threading

_td = threading.Thread(target=lambda: None, daemon=True)
assert _td.daemon, "daemon thread is daemon"
_tn = threading.Thread(target=lambda: None, daemon=False)
assert not _tn.daemon, "non-daemon thread not daemon"

print("daemon_flag_roundtrip OK")
"###);
    assert_output(&out, r###"daemon_flag_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/enumerate_tracks_live_threads.py`.
#[test]
fn test_gen_behavior_std_libs_threading_enumerate_tracks_live_threads() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "enumerate_tracks_live_threads"
# subject = "threading.enumerate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.enumerate: a running thread is a member of enumerate(); after it is joined it is no longer enumerated"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

held = threading.Thread(target=hold)
held.start()
assert held in threading.enumerate(), "running thread is enumerated"
gate.set()
held.join()
assert held not in threading.enumerate(), "joined thread is not enumerated"

print("enumerate_tracks_live_threads OK")
"###);
    assert_output(&out, r###"enumerate_tracks_live_threads OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/event_clear_resets_is_set.py`.
#[test]
fn test_gen_behavior_std_libs_threading_event_clear_resets_is_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_clear_resets_is_set"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: Event.set() makes is_set() true; Event.clear() resets is_set() to false"""
import threading

_ev = threading.Event()
_ev.set()
assert _ev.is_set(), "event set after set()"
_ev.clear()
assert not _ev.is_set(), "event unset after clear()"

print("event_clear_resets_is_set OK")
"###);
    assert_output(&out, r###"event_clear_resets_is_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/event_initially_unset.py`.
#[test]
fn test_gen_behavior_std_libs_threading_event_initially_unset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_initially_unset"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: a freshly constructed Event reports is_set() == False"""
import threading

_ev = threading.Event()
assert not _ev.is_set(), "a fresh Event is initially unset"

print("event_initially_unset OK")
"###);
    assert_output(&out, r###"event_initially_unset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/event_wait_returns_after_set.py`.
#[test]
fn test_gen_behavior_std_libs_threading_event_wait_returns_after_set() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "event_wait_returns_after_set"
# subject = "threading.Event"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Event: Event.wait() in a worker returns only after the main thread calls set()"""
import threading

import time

_ev = threading.Event()
_ev_result = []

def _waiter():
    _ev.wait()
    _ev_result.append("done")

tw = threading.Thread(target=_waiter)
tw.start()
time.sleep(0.01)
_ev.set()
tw.join()
assert _ev_result == ["done"], f"event result = {_ev_result!r}"

print("event_wait_returns_after_set OK")
"###);
    assert_output(&out, r###"event_wait_returns_after_set OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/excepthook_default_restorable.py`.
#[test]
fn test_gen_behavior_std_libs_threading_excepthook_default_restorable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_default_restorable"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: threading.__excepthook__ is the preserved default and excepthook can be restored to it after temporary replacement"""
import threading

original = threading.excepthook
threading.excepthook = lambda args: None
threading.excepthook = original
assert threading.__excepthook__ is not None, "default excepthook present"
assert threading.excepthook is original, "excepthook restored"

print("excepthook_default_restorable OK")
"###);
    assert_output(&out, r###"excepthook_default_restorable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/excepthook_receives_uncaught.py`.
#[test]
fn test_gen_behavior_std_libs_threading_excepthook_receives_uncaught() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_receives_uncaught"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: an exception escaping a worker's run() is delivered to a custom threading.excepthook (exc_type/exc_value/thread.name), not re-raised in the joiner"""
import threading

captured = []

def hook(args):
    captured.append((args.exc_type.__name__, str(args.exc_value), args.thread.name))

original = threading.excepthook
threading.excepthook = hook
try:
    def boom():
        raise ValueError("boom in thread")

    t = threading.Thread(target=boom, name="boomer")
    t.start()
    t.join()  # join itself does NOT re-raise the worker exception
finally:
    threading.excepthook = original

assert len(captured) == 1, f"hook calls = {captured!r}"
exc_name, exc_msg, thread_name = captured[0]
assert exc_name == "ValueError", f"exc type = {exc_name!r}"
assert exc_msg == "boom in thread", f"exc msg = {exc_msg!r}"
assert thread_name == "boomer", f"thread name = {thread_name!r}"

print("excepthook_receives_uncaught OK")
"###);
    assert_output(&out, r###"excepthook_receives_uncaught OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/excepthook_skipped_when_handled.py`.
#[test]
fn test_gen_behavior_std_libs_threading_excepthook_skipped_when_handled() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "excepthook_skipped_when_handled"
# subject = "threading.excepthook"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.excepthook: a Thread subclass that catches its own exception inside run() never reaches threading.excepthook"""
import threading

original = threading.excepthook
seen = []
threading.excepthook = lambda args: seen.append(args.exc_type)
try:
    class Caught(threading.Thread):
        def __init__(self):
            super().__init__()
            self.exc = None
        def run(self):
            try:
                raise RuntimeError("handled")
            except RuntimeError as e:
                self.exc = e

    c = Caught()
    c.start()
    c.join()
finally:
    threading.excepthook = original

assert seen == [], "hook not called when run() handles its own exception"
assert isinstance(c.exc, RuntimeError), f"caught exc = {c.exc!r}"

print("excepthook_skipped_when_handled OK")
"###);
    assert_output(&out, r###"excepthook_skipped_when_handled OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/explicit_name_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_threading_explicit_name_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "explicit_name_preserved"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: an explicit name='my-thread' is preserved verbatim even with a target, and is settable after construction"""
import threading

def worker():
    pass

explicit = threading.Thread(target=worker, name="my-thread")
assert explicit.name == "my-thread", f"explicit name = {explicit.name!r}"
explicit.name = "renamed"
assert explicit.name == "renamed", f"renamed = {explicit.name!r}"

print("explicit_name_preserved OK")
"###);
    assert_output(&out, r###"explicit_name_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/falsy_callable_target_is_invoked.py`.
#[test]
fn test_gen_behavior_std_libs_threading_falsy_callable_target_is_invoked() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "falsy_callable_target_is_invoked"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: the target is checked for 'is not None', not truthiness, so a callable whose __bool__ is False is still invoked"""
import threading

class _FalsyCallable:
    def __init__(self):
        self.ran = False
    def __bool__(self):
        return False
    def __call__(self):
        self.ran = True

_falsy = _FalsyCallable()
tf = threading.Thread(target=_falsy)
tf.start()
tf.join()
assert _falsy.ran, "falsy callable target was invoked"

print("falsy_callable_target_is_invoked OK")
"###);
    assert_output(&out, r###"falsy_callable_target_is_invoked OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/ident_set_after_start.py`.
#[test]
fn test_gen_behavior_std_libs_threading_ident_set_after_start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "ident_set_after_start"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.ident is None before start() and becomes a non-None integer after the thread has run"""
import threading

fresh = threading.Thread(target=lambda: None)
assert fresh.ident is None, f"unstarted ident = {fresh.ident!r}"
fresh.start()
fresh.join()
assert fresh.ident is not None, f"started ident = {fresh.ident!r}"
assert isinstance(fresh.ident, int), f"started ident type = {type(fresh.ident)!r}"

print("ident_set_after_start OK")
"###);
    assert_output(&out, r###"ident_set_after_start OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/is_alive_lifecycle.py`.
#[test]
fn test_gen_behavior_std_libs_threading_is_alive_lifecycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "is_alive_lifecycle"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.is_alive() is False before start, True while running, and False again after join"""
import threading

import time

_ta = threading.Thread(target=lambda: time.sleep(0.02))
assert not _ta.is_alive(), "before start: not alive"
_ta.start()
assert _ta.is_alive(), "during run: alive"
_ta.join()
assert not _ta.is_alive(), "after join: not alive"

print("is_alive_lifecycle OK")
"###);
    assert_output(&out, r###"is_alive_lifecycle OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/join_blocks_until_finished.py`.
#[test]
fn test_gen_behavior_std_libs_threading_join_blocks_until_finished() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "join_blocks_until_finished"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread.join() blocks until the worker finishes, so a worker append precedes the post-join main append in order"""
import threading

import time

_order = []

def _slow():
    time.sleep(0.01)
    _order.append("thread")

t = threading.Thread(target=_slow)
t.start()
t.join()
_order.append("main")
assert _order == ["thread", "main"], f"join order = {_order!r}"

print("join_blocks_until_finished OK")
"###);
    assert_output(&out, r###"join_blocks_until_finished OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/local_attribute_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_threading_local_attribute_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "local_attribute_roundtrip"
# subject = "threading.local"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.local: an attribute assigned on a threading.local() instance reads back equal in the same thread"""
import threading

_local = threading.local()
_local.x = 42
assert _local.x == 42, f"thread local x = {_local.x!r}"

print("local_attribute_roundtrip OK")
"###);
    assert_output(&out, r###"local_attribute_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/lock_serializes_increments.py`.
#[test]
fn test_gen_behavior_std_libs_threading_lock_serializes_increments() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "lock_serializes_increments"
# subject = "threading.Lock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock: a shared Lock serializes 5 threads x 100 increments of a shared counter to exactly 500"""
import threading

_counter = [0]
_lock = threading.Lock()

def _increment():
    for _ in range(100):
        with _lock:
            _counter[0] += 1

_threads = [threading.Thread(target=_increment) for _ in range(5)]
for _th in _threads:
    _th.start()
for _th in _threads:
    _th.join()
assert _counter[0] == 500, f"locked counter = {_counter[0]!r}"

print("lock_serializes_increments OK")
"###);
    assert_output(&out, r###"lock_serializes_increments OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/main_thread_named_mainthread.py`.
#[test]
fn test_gen_behavior_std_libs_threading_main_thread_named_mainthread() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "main_thread_named_mainthread"
# subject = "threading.main_thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.main_thread: main_thread() is named 'MainThread' and its ident equals current_thread().ident and get_ident() on the main thread"""
import threading

main = threading.main_thread()
assert main.name == "MainThread", f"main name = {main.name!r}"
assert main.ident == threading.current_thread().ident, "main ident == current ident"
assert main.ident == threading.get_ident(), "main ident == get_ident()"

print("main_thread_named_mainthread OK")
"###);
    assert_output(&out, r###"main_thread_named_mainthread OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/repr_lifecycle_markers.py`.
#[test]
fn test_gen_behavior_std_libs_threading_repr_lifecycle_markers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "repr_lifecycle_markers"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: repr() shows the lifecycle markers 'initial' before start, 'started' while running, and 'stopped' after join"""
import threading

gate = threading.Event()

def hold():
    gate.wait()

live = threading.Thread(target=hold)
assert "initial" in repr(live), f"initial repr = {repr(live)!r}"
live.start()
assert "started" in repr(live), f"started repr = {repr(live)!r}"
gate.set()
live.join()
assert "stopped" in repr(live), f"stopped repr = {repr(live)!r}"

print("repr_lifecycle_markers OK")
"###);
    assert_output(&out, r###"repr_lifecycle_markers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/repr_shows_daemon.py`.
#[test]
fn test_gen_behavior_std_libs_threading_repr_shows_daemon() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "repr_shows_daemon"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: repr() of a fresh thread omits 'daemon'; after setting daemon=True repr() includes 'daemon'"""
import threading

fresh = threading.Thread()
assert "daemon" not in repr(fresh), f"fresh repr = {repr(fresh)!r}"
fresh.daemon = True
assert "daemon" in repr(fresh), f"daemon repr = {repr(fresh)!r}"

print("repr_shows_daemon OK")
"###);
    assert_output(&out, r###"repr_shows_daemon OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/semaphore_limits_concurrency.py`.
#[test]
fn test_gen_behavior_std_libs_threading_semaphore_limits_concurrency() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "semaphore_limits_concurrency"
# subject = "threading.Semaphore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Semaphore: Semaphore(2) grants two non-blocking acquires, refuses the third, and grants again after a release"""
import threading

_sem = threading.Semaphore(2)
assert _sem.acquire(blocking=False), "first acquire succeeds"
assert _sem.acquire(blocking=False), "second acquire succeeds"
assert not _sem.acquire(blocking=False), "third acquire fails (limit=2)"
_sem.release()
assert _sem.acquire(blocking=False), "after release, acquire succeeds again"
_sem.release()
_sem.release()

print("semaphore_limits_concurrency OK")
"###);
    assert_output(&out, r###"semaphore_limits_concurrency OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/subclass_run_override.py`.
#[test]
fn test_gen_behavior_std_libs_threading_subclass_run_override() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "subclass_run_override"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: subclassing Thread and overriding run() (no target) runs the overridden body on start()"""
import threading

class _Counter(threading.Thread):
    def __init__(self):
        super().__init__()
        self.total = 0
    def run(self):
        for _ in range(10):
            self.total += 1

ct = _Counter()
ct.start()
ct.join()
assert ct.total == 10, f"subclass run() total = {ct.total!r}"

print("subclass_run_override OK")
"###);
    assert_output(&out, r###"subclass_run_override OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/target_name_appended.py`.
#[test]
fn test_gen_behavior_std_libs_threading_target_name_appended() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "target_name_appended"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: Thread(target=worker) auto-name starts with 'Thread-' and ends with the target's name in parens, '(worker)'"""
import threading

def worker():
    pass

named_target = threading.Thread(target=worker)
assert named_target.name.startswith("Thread-"), named_target.name
assert named_target.name.endswith("(worker)"), f"target name = {named_target.name!r}"

print("target_name_appended OK")
"###);
    assert_output(&out, r###"target_name_appended OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/thread_local_isolation.py`.
#[test]
fn test_gen_behavior_std_libs_threading_thread_local_isolation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_local_isolation"
# subject = "threading.local"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.local: a value stored on threading.local() in the main thread is not visible in a worker thread (per-thread storage isolation)"""
import threading

_local = threading.local()
_local.val = "main"
_result = []

def _check_local():
    # A new thread starts without _local.val.
    try:
        _ = _local.val
        _result.append("found")
    except AttributeError:
        _result.append("not found")

tl = threading.Thread(target=_check_local)
tl.start()
tl.join()
assert _result == ["not found"], f"thread local isolation = {_result!r}"

print("thread_local_isolation OK")
"###);
    assert_output(&out, r###"thread_local_isolation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/thread_runs_target.py`.
#[test]
fn test_gen_behavior_std_libs_threading_thread_runs_target() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "thread_runs_target"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: a Thread runs its target function and the side effect is visible after join()"""
import threading

_result = []

def _worker():
    _result.append(42)

t = threading.Thread(target=_worker)
t.start()
t.join()
assert _result == [42], f"thread result = {_result!r}"

print("thread_runs_target OK")
"###);
    assert_output(&out, r###"thread_runs_target OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/threading/worker_ident_differs_from_main.py`.
#[test]
fn test_gen_behavior_std_libs_threading_worker_ident_differs_from_main() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "worker_ident_differs_from_main"
# subject = "threading.get_ident"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.get_ident: a worker thread sees get_ident() == current_thread().ident, both distinct from the main thread's ident"""
import threading

main = threading.main_thread()
idents = {}

def record():
    idents["worker"] = threading.get_ident()
    idents["current"] = threading.current_thread().ident

w = threading.Thread(target=record)
w.start()
w.join()
assert idents["worker"] == idents["current"], "get_ident == current_thread().ident in worker"
assert idents["worker"] != main.ident, "worker ident differs from main"

print("worker_ident_differs_from_main OK")
"###);
    assert_output(&out, r###"worker_ident_differs_from_main OK
"###);
}
