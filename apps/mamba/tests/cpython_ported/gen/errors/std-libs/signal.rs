use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/signal/pthread_sigmask_bad_how_oserror.py`.
#[test]
fn test_gen_errors_std_libs_signal_pthread_sigmask_bad_how_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_bad_how_oserror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_bad_how_oserror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(1700, [])
except OSError:
    _raised = True
assert _raised, "pthread_sigmask_bad_how_oserror: expected OSError"
print("pthread_sigmask_bad_how_oserror OK")
"###);
    assert_output(&out, r###"pthread_sigmask_bad_how_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/pthread_sigmask_huge_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_pthread_sigmask_huge_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_huge_valueerror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_huge_valueerror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(signal.SIG_BLOCK, [1 << 1000])
except ValueError:
    _raised = True
assert _raised, "pthread_sigmask_huge_valueerror: expected ValueError"
print("pthread_sigmask_huge_valueerror OK")
"###);
    assert_output(&out, r###"pthread_sigmask_huge_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/pthread_sigmask_nsig_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_pthread_sigmask_nsig_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_nsig_valueerror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_nsig_valueerror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(signal.SIG_BLOCK, [signal.NSIG])
except ValueError:
    _raised = True
assert _raised, "pthread_sigmask_nsig_valueerror: expected ValueError"
print("pthread_sigmask_nsig_valueerror OK")
"###);
    assert_output(&out, r###"pthread_sigmask_nsig_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/pthread_sigmask_zero_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_pthread_sigmask_zero_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "pthread_sigmask_zero_valueerror"
# subject = "signal.pthread_sigmask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.pthread_sigmask: pthread_sigmask_zero_valueerror (errors)."""
import signal

_raised = False
try:
    signal.pthread_sigmask(signal.SIG_BLOCK, [0])
except ValueError:
    _raised = True
assert _raised, "pthread_sigmask_zero_valueerror: expected ValueError"
print("pthread_sigmask_zero_valueerror OK")
"###);
    assert_output(&out, r###"pthread_sigmask_zero_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/set_wakeup_fd_bad_descriptor_raises.py`.
#[test]
fn test_gen_errors_std_libs_signal_set_wakeup_fd_bad_descriptor_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "set_wakeup_fd_bad_descriptor_raises"
# subject = "signal.set_wakeup_fd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.set_wakeup_fd: set_wakeup_fd_bad_descriptor_raises (errors)."""
import signal

_raised = False
try:
    signal.set_wakeup_fd(2 ** 30)
except (ValueError, OSError):
    _raised = True
assert _raised, "set_wakeup_fd_bad_descriptor_raises: expected (ValueError, OSError)"
print("set_wakeup_fd_bad_descriptor_raises OK")
"###);
    assert_output(&out, r###"set_wakeup_fd_bad_descriptor_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_non_callable_handler_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_non_callable_handler_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_non_callable_handler_typeerror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_non_callable_handler_typeerror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGUSR1, 42)
except TypeError:
    _raised = True
assert _raised, "signal_non_callable_handler_typeerror: expected TypeError"
print("signal_non_callable_handler_typeerror OK")
"###);
    assert_output(&out, r###"signal_non_callable_handler_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_non_int_signum_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_non_int_signum_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_non_int_signum_typeerror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_non_int_signum_typeerror (errors)."""
import signal

_raised = False
try:
    signal.signal('not_a_signum', signal.SIG_IGN)
except TypeError:
    _raised = True
assert _raised, "signal_non_int_signum_typeerror: expected TypeError"
print("signal_non_int_signum_typeerror OK")
"###);
    assert_output(&out, r###"signal_non_int_signum_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_none_handler_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_none_handler_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_none_handler_typeerror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_none_handler_typeerror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGUSR1, None)
except TypeError:
    _raised = True
assert _raised, "signal_none_handler_typeerror: expected TypeError"
print("signal_none_handler_typeerror OK")
"###);
    assert_output(&out, r###"signal_none_handler_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_out_of_range_signum_raises.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_out_of_range_signum_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_out_of_range_signum_raises"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_out_of_range_signum_raises (errors)."""
import signal

_raised = False
try:
    signal.signal(99999, signal.SIG_IGN)
except (ValueError, OSError):
    _raised = True
assert _raised, "signal_out_of_range_signum_raises: expected (ValueError, OSError)"
print("signal_out_of_range_signum_raises OK")
"###);
    assert_output(&out, r###"signal_out_of_range_signum_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_sigkill_oserror.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_sigkill_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_sigkill_oserror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_sigkill_oserror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGKILL, signal.SIG_IGN)
except OSError:
    _raised = True
assert _raised, "signal_sigkill_oserror: expected OSError"
print("signal_sigkill_oserror OK")
"###);
    assert_output(&out, r###"signal_sigkill_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/signal/signal_sigstop_oserror.py`.
#[test]
fn test_gen_errors_std_libs_signal_signal_sigstop_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "errors"
# case = "signal_sigstop_oserror"
# subject = "signal.signal"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal_sigstop_oserror (errors)."""
import signal

_raised = False
try:
    signal.signal(signal.SIGSTOP, signal.SIG_IGN)
except OSError:
    _raised = True
assert _raised, "signal_sigstop_oserror: expected OSError"
print("signal_sigstop_oserror OK")
"###);
    assert_output(&out, r###"signal_sigstop_oserror OK
"###);
}
