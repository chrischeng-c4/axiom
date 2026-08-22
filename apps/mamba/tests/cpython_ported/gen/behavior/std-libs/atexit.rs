use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/atexit/ncallbacks_counts_registrations.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_ncallbacks_counts_registrations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "ncallbacks_counts_registrations"
# subject = "atexit._ncallbacks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._ncallbacks: _clear()/register()/_ncallbacks() track the queue length: 0 after clear, increments per register"""
import atexit


def cleanup1():
    pass


def cleanup2():
    pass


atexit._clear()
assert atexit._ncallbacks() == 0, "queue empty after _clear()"
atexit.register(cleanup1)
assert atexit._ncallbacks() == 1, "one registration counted"
atexit.register(cleanup2)
assert atexit._ncallbacks() == 2, "two registrations counted"
atexit._clear()
assert atexit._ncallbacks() == 0, "queue empty after second _clear()"
print("ncallbacks_counts_registrations OK")
"###);
    assert_output(&out, r###"ncallbacks_counts_registrations OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/register_bound_method_fires.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_register_bound_method_fires() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_bound_method_fires"
# subject = "atexit.register"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: a bound method registered with an argument fires at _run_exitfuncs() like any other callable"""
import atexit

atexit._clear()
collected = []
atexit.register(collected.append, 5)
atexit._run_exitfuncs()
assert collected == [5], f"bound method fired with arg: {collected}"
atexit._clear()
print("register_bound_method_fires OK")
"###);
    assert_output(&out, r###"register_bound_method_fires OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/register_duplicate_fires_twice.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_register_duplicate_fires_twice() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_duplicate_fires_twice"
# subject = "atexit.register"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: the same callable registered twice fires twice at _run_exitfuncs()"""
import atexit

atexit._clear()
hits = []
atexit.register(hits.append, "x")
atexit.register(hits.append, "x")
atexit._run_exitfuncs()
assert hits == ["x", "x"], f"duplicate registration fires twice: {hits}"
atexit._clear()
print("register_duplicate_fires_twice OK")
"###);
    assert_output(&out, r###"register_duplicate_fires_twice OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/register_returns_registered_callable.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_register_returns_registered_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "register_returns_registered_callable"
# subject = "atexit.register"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: register() returns the callable it was given (identity), the documented return contract"""
import atexit


def cleanup():
    pass


atexit._clear()
ret = atexit.register(cleanup)
assert ret is cleanup, f"register should return the callable: {ret!r}"
atexit._clear()
print("register_returns_registered_callable OK")
"###);
    assert_output(&out, r###"register_returns_registered_callable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/run_after_clear_runs_nothing.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_run_after_clear_runs_nothing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_after_clear_runs_nothing"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() after _clear() runs nothing and leaves an empty queue"""
import atexit

fired = []


def cleanup():
    fired.append(1)


atexit._clear()
atexit.register(cleanup)
atexit._clear()
result = atexit._run_exitfuncs()
assert result is None, f"_run_exitfuncs() returns None: {result!r}"
assert fired == [], f"cleared callback must not fire: {fired}"
assert atexit._ncallbacks() == 0, "queue stays empty"
print("run_after_clear_runs_nothing OK")
"###);
    assert_output(&out, r###"run_after_clear_runs_nothing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/run_exitfuncs_returns_none_and_drains.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_run_exitfuncs_returns_none_and_drains() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_exitfuncs_returns_none_and_drains"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() returns None and drains the queue so _ncallbacks() is 0 afterwards"""
import atexit


def cleanup():
    pass


atexit._clear()
atexit.register(cleanup)
assert atexit._ncallbacks() == 1, "one callback registered"
result = atexit._run_exitfuncs()
assert result is None, f"_run_exitfuncs() returns None: {result!r}"
assert atexit._ncallbacks() == 0, "queue drained after running"
atexit._clear()
print("run_exitfuncs_returns_none_and_drains OK")
"###);
    assert_output(&out, r###"run_exitfuncs_returns_none_and_drains OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/run_one_shot_drain.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_run_one_shot_drain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_one_shot_drain"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: _run_exitfuncs() runs each callback exactly once; a second call is a no-op because the queue is already drained"""
import atexit

once = []


def mark():
    once.append(1)


atexit._clear()
atexit.register(mark)
atexit._run_exitfuncs()
atexit._run_exitfuncs()
assert once == [1], f"callback fires exactly once across two runs: {once}"
atexit._clear()
print("run_one_shot_drain OK")
"###);
    assert_output(&out, r###"run_one_shot_drain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/run_raising_callback_is_isolated.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_run_raising_callback_is_isolated() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_raising_callback_is_isolated"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: a callback that raises does not abort the run: the exception is reported to stderr and remaining callbacks still execute"""
import atexit
import contextlib
import io

order = []


def good():
    order.append("good")


def bad():
    raise ValueError("boom")


atexit._clear()
atexit.register(good)  # runs second (LIFO)
atexit.register(bad)   # runs first, raises
buf = io.StringIO()
with contextlib.redirect_stderr(buf):
    atexit._run_exitfuncs()
report = buf.getvalue()
assert order == ["good"], f"survivor still ran: {order}"
assert "ValueError" in report, f"exception reported to stderr: {report!r}"
atexit._clear()
print("run_raising_callback_is_isolated OK")
"###);
    assert_output(&out, r###"run_raising_callback_is_isolated OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/run_stress_128_each_once.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_run_stress_128_each_once() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "run_stress_128_each_once"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: 128 registered callbacks each fire exactly once and the queue drains to empty"""
import atexit

counter = [0]


def bump():
    counter[0] += 1


atexit._clear()
for _ in range(128):
    atexit.register(bump)
atexit._run_exitfuncs()
assert counter[0] == 128, f"every callback fired once: {counter[0]}"
assert atexit._ncallbacks() == 0, "queue drained after run"
atexit._clear()
print("run_stress_128_each_once OK")
"###);
    assert_output(&out, r###"run_stress_128_each_once OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/unregister_by_identity_net_effect.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_unregister_by_identity_net_effect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_by_identity_net_effect"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = "unregister matches by string name not callable identity, and _run_exitfuncs() never fires handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() matches by callable identity: dropping one of two distinct registered callables leaves only the other to fire"""
import atexit

a = [0]


def inc():
    a[0] += 1


def dec():
    a[0] -= 1


atexit._clear()
for _ in range(4):
    atexit.register(inc)
atexit.register(dec)
atexit.unregister(inc)  # drops all four inc registrations by identity
atexit._run_exitfuncs()
assert a[0] == -1, f"only dec survived: {a[0]}"
atexit._clear()
print("unregister_by_identity_net_effect OK")
"###);
    assert_output(&out, r###"unregister_by_identity_net_effect OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/unregister_missing_is_silent.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_unregister_missing_is_silent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_missing_is_silent"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() of a callable that was never registered is a silent no-op (no raise), queue length unchanged"""
import atexit


def never_registered():
    pass


atexit._clear()
# No raise even though `never_registered` is not in the queue.
atexit.unregister(never_registered)
assert atexit._ncallbacks() == 0, "queue length unchanged by no-op unregister"
atexit._clear()
print("unregister_missing_is_silent OK")
"###);
    assert_output(&out, r###"unregister_missing_is_silent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/atexit/unregister_removes_all_copies.py`.
#[test]
fn test_gen_behavior_std_libs_atexit_unregister_removes_all_copies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "unregister_removes_all_copies"
# subject = "atexit.unregister"
# kind = "semantic"
# xfail = "unregister matches by string name not callable identity, and _run_exitfuncs() never fires handlers (stub, #652)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.unregister: unregister() removes every copy of a duplicated registration so the callback fires zero times"""
import atexit

fired = []


def note():
    fired.append(1)


atexit._clear()
atexit.register(note)
atexit.register(note)
atexit.unregister(note)  # cancels BOTH copies
atexit._run_exitfuncs()
# The observable contract: every copy is cancelled, so the callback fires
# zero times.
assert fired == [], f"removed callback must not fire: {fired}"
atexit._clear()
print("unregister_removes_all_copies OK")
"###);
    assert_output(&out, r###"unregister_removes_all_copies OK
"###);
}
