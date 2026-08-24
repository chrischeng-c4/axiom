use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/signal/alarm_zero_returns_zero.py`.
#[test]
fn test_gen_behavior_std_libs_signal_alarm_zero_returns_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "alarm_zero_returns_zero"
# subject = "signal.alarm"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.alarm: alarm(0) cancels any pending alarm and returns 0 (no previous alarm was scheduled)"""
import signal

# No alarm has been scheduled, so cancelling returns the 0-second remainder.
assert signal.alarm(0) == 0, "alarm(0) returns 0 when none scheduled"
print("alarm_zero_returns_zero OK")
"###);
    assert_output(&out, r###"alarm_zero_returns_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/generic_tests__test_functions_module_attr.py`.
#[test]
fn test_gen_behavior_std_libs_signal_generic_tests__test_functions_module_attr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "generic_tests__test_functions_module_attr"
# subject = "cpython.test_signal.GenericTests.test_functions_module_attr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::GenericTests::test_functions_module_attr
"""Auto-ported test: GenericTests::test_functions_module_attr (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
for name in dir(signal):
    value = getattr(signal, name)
    if inspect.isroutine(value) and (not inspect.isbuiltin(value)):

        assert value.__module__ == 'signal'
print("GenericTests::test_functions_module_attr: ok")
"###);
    assert_output(&out, r###"GenericTests::test_functions_module_attr: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/handler_fires_once_per_kill.py`.
#[test]
fn test_gen_behavior_std_libs_signal_handler_fires_once_per_kill() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "handler_fires_once_per_kill"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: a counting SIGUSR1 handler fires once per os.kill so three kills increment the counter to exactly 3"""
import os
import signal

_count = [0]


def _multi_handler(signum, frame):
    _count[0] += 1


signal.signal(signal.SIGUSR1, _multi_handler)
os.kill(os.getpid(), signal.SIGUSR1)
os.kill(os.getpid(), signal.SIGUSR1)
os.kill(os.getpid(), signal.SIGUSR1)
assert _count[0] == 3, f"handler called 3 times: {_count[0]!r}"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("handler_fires_once_per_kill OK")
"###);
    assert_output(&out, r###"handler_fires_once_per_kill OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/handler_receives_signum.py`.
#[test]
fn test_gen_behavior_std_libs_signal_handler_receives_signum() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "handler_receives_signum"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: a custom SIGUSR1 handler installed via signal.signal runs exactly once per os.kill(getpid(), SIGUSR1) and receives the signal number as its first argument"""
import os
import signal

_received = []


def _record_handler(signum, frame):
    _received.append(signum)


signal.signal(signal.SIGUSR1, _record_handler)
os.kill(os.getpid(), signal.SIGUSR1)
assert len(_received) == 1, f"handler called once: {_received!r}"
assert _received[0] == signal.SIGUSR1, f"signum = {_received[0]!r}"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("handler_receives_signum OK")
"###);
    assert_output(&out, r###"handler_receives_signum OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/posix_tests__test_setting_signal_handler_to_none_raises_error.py`.
#[test]
fn test_gen_behavior_std_libs_signal_posix_tests__test_setting_signal_handler_to_none_raises_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "posix_tests__test_setting_signal_handler_to_none_raises_error"
# subject = "cpython.test_signal.PosixTests.test_setting_signal_handler_to_none_raises_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::PosixTests::test_setting_signal_handler_to_none_raises_error
"""Auto-ported test: PosixTests::test_setting_signal_handler_to_none_raises_error (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---

try:
    signal.signal(signal.SIGUSR1, None)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("PosixTests::test_setting_signal_handler_to_none_raises_error: ok")
"###);
    assert_output(&out, r###"PosixTests::test_setting_signal_handler_to_none_raises_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/raise_signal_custom_handler_routing.py`.
#[test]
fn test_gen_behavior_std_libs_signal_raise_signal_custom_handler_routing() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "raise_signal_custom_handler_routing"
# subject = "signal.raise_signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.raise_signal: a custom SIGINT handler intercepts raise_signal(SIGINT); each call routes to the handler and raise_signal itself returns None"""
import signal

seen = []
signal.signal(signal.SIGINT, lambda s, f: seen.append(s))

signal.raise_signal(signal.SIGINT)
assert seen == [signal.SIGINT], f"custom handler saw {seen!r}"

# raise_signal returns None on success and the handler fires a second time.
result = signal.raise_signal(signal.SIGINT)
assert result is None, f"raise_signal returns None, got {result!r}"
assert seen == [signal.SIGINT, signal.SIGINT], f"handler fired twice: {seen!r}"

signal.signal(signal.SIGINT, signal.default_int_handler)
print("raise_signal_custom_handler_routing OK")
"###);
    assert_output(&out, r###"raise_signal_custom_handler_routing OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/raise_signal_default_int_handler_keyboardinterrupt.py`.
#[test]
fn test_gen_behavior_std_libs_signal_raise_signal_default_int_handler_keyboardinterrupt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "raise_signal_default_int_handler_keyboardinterrupt"
# subject = "signal.raise_signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.raise_signal: with default_int_handler installed for SIGINT, raise_signal(SIGINT) surfaces synchronously as KeyboardInterrupt"""
import signal

signal.signal(signal.SIGINT, signal.default_int_handler)
hit_kbd = False
try:
    signal.raise_signal(signal.SIGINT)
except KeyboardInterrupt:
    hit_kbd = True
assert hit_kbd, "raise_signal(SIGINT) raises KeyboardInterrupt"

signal.signal(signal.SIGINT, signal.default_int_handler)
print("raise_signal_default_int_handler_keyboardinterrupt OK")
"###);
    assert_output(&out, r###"raise_signal_default_int_handler_keyboardinterrupt OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/sig_ign_suppresses_delivery.py`.
#[test]
fn test_gen_behavior_std_libs_signal_sig_ign_suppresses_delivery() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "sig_ign_suppresses_delivery"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: installing SIG_IGN for SIGUSR2 makes os.kill(getpid(), SIGUSR2) a no-op (handler never runs) and getsignal still reports SIG_IGN; then restore SIG_DFL"""
import os
import signal

signal.signal(signal.SIGUSR2, signal.SIG_IGN)
os.kill(os.getpid(), signal.SIGUSR2)  # ignored: must not raise or run anything
assert signal.getsignal(signal.SIGUSR2) == signal.SIG_IGN, "still SIG_IGN"

signal.signal(signal.SIGUSR2, signal.SIG_DFL)
print("sig_ign_suppresses_delivery OK")
"###);
    assert_output(&out, r###"sig_ign_suppresses_delivery OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/signal_returns_previous_handler.py`.
#[test]
fn test_gen_behavior_std_libs_signal_signal_returns_previous_handler() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "signal_returns_previous_handler"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal.signal returns the handler it replaced; installing a second handler over SIGUSR1 returns the first one, then restore SIG_DFL"""
import signal


def _handler_a(n, f):
    pass


def _handler_b(n, f):
    pass


# The first install returns whatever was registered before (SIG_DFL/SIG_IGN
# or some callable inherited from the runner).
_prev1 = signal.signal(signal.SIGUSR1, _handler_a)
assert _prev1 in (signal.SIG_DFL, signal.SIG_IGN) or callable(_prev1), \
    f"first signal: {_prev1!r}"

# The second install must return exactly the handler we just put in place.
_prev2 = signal.signal(signal.SIGUSR1, _handler_b)
assert _prev2 is _handler_a, "second signal returns previous handler"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("signal_returns_previous_handler OK")
"###);
    assert_output(&out, r###"signal_returns_previous_handler OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/signals_enum_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_signal_signals_enum_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "signals_enum_roundtrip"
# subject = "signal.Signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.Signals: signal constants are typed Signals/Handlers enum members (SIGINT/SIGTERM are Signals, SIG_DFL/SIG_IGN are Handlers); Signals(2) is SIGINT and Signals(SIGINT).name == 'SIGINT'"""
import signal

# Constants are typed enum members, not bare ints.
assert isinstance(signal.SIGINT, signal.Signals), "SIGINT is Signals member"
assert isinstance(signal.SIGTERM, signal.Signals), "SIGTERM is Signals member"
assert isinstance(signal.SIG_DFL, signal.Handlers), "SIG_DFL is Handlers member"
assert isinstance(signal.SIG_IGN, signal.Handlers), "SIG_IGN is Handlers member"

# Signals members round-trip through their integer value.
assert signal.Signals(2) is signal.SIGINT, "Signals(2) is SIGINT"
assert signal.Signals(signal.SIGINT).name == "SIGINT", "Signals name lookup"

print("signals_enum_roundtrip OK")
"###);
    assert_output(&out, r###"signals_enum_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/sigterm_sigint_constant_values.py`.
#[test]
fn test_gen_behavior_std_libs_signal_sigterm_sigint_constant_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "sigterm_sigint_constant_values"
# subject = "signal.Signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.Signals: POSIX-stable signal numbers: SIGTERM == 15, SIGINT == 2, SIGKILL == 9, SIGHUP == 1, SIGALRM == 14, SIG_DFL == 0, SIG_IGN == 1"""
import signal

for name, value in [
    ("SIGINT", 2),
    ("SIGTERM", 15),
    ("SIGKILL", 9),
    ("SIGHUP", 1),
    ("SIGALRM", 14),
    ("SIG_DFL", 0),
    ("SIG_IGN", 1),
]:
    got = int(getattr(signal, name))
    assert got == value, f"{name} = {got!r}, expected {value}"

print("sigterm_sigint_constant_values OK")
"###);
    assert_output(&out, r###"sigterm_sigint_constant_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/strsignal_keyword_descriptions.py`.
#[test]
fn test_gen_behavior_std_libs_signal_strsignal_keyword_descriptions() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "strsignal_keyword_descriptions"
# subject = "signal.strsignal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.strsignal: strsignal returns the OS description string carrying a stable keyword per signal (Interrupt for SIGINT, Terminated for SIGTERM, Hangup for SIGHUP) and accepts the raw int just like the enum member"""
import signal

# The exact text varies by platform (some append the number, e.g.
# "Interrupt: 2"), so assert on the stable English keyword, not the whole text.
sigint_desc = signal.strsignal(signal.SIGINT)
assert isinstance(sigint_desc, str), f"SIGINT desc type = {type(sigint_desc)!r}"
assert "Interrupt" in sigint_desc, f"SIGINT desc = {sigint_desc!r}"

assert "Terminated" in signal.strsignal(signal.SIGTERM), "SIGTERM keyword"
assert "Hangup" in signal.strsignal(signal.SIGHUP), "SIGHUP keyword"

# strsignal accepts the raw integer just as well as the enum member.
assert signal.strsignal(int(signal.SIGINT)) == sigint_desc, "int matches enum"

print("strsignal_keyword_descriptions OK")
"###);
    assert_output(&out, r###"strsignal_keyword_descriptions OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/valid_signals_set_contents.py`.
#[test]
fn test_gen_behavior_std_libs_signal_valid_signals_set_contents() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "valid_signals_set_contents"
# subject = "signal.valid_signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.valid_signals: valid_signals() returns a set holding SIGTERM and SIGINT but excluding the 0 and NSIG boundary markers, smaller than NSIG and with many entries"""
import signal

vs = signal.valid_signals()
assert isinstance(vs, (set, frozenset)), f"valid_signals type = {type(vs)!r}"
assert signal.SIGTERM in vs, "SIGTERM in valid_signals"
assert signal.SIGINT in vs, "SIGINT in valid_signals"

# Boundaries: 0 is not a signal, NSIG is the past-the-end marker.
assert 0 not in vs, "0 not in valid_signals"
assert signal.NSIG not in vs, "NSIG not in valid_signals"
assert len(vs) < signal.NSIG, "valid_signals smaller than NSIG"
assert len(vs) >= 6, f"valid_signals has many entries: {len(vs)}"

print("valid_signals_set_contents OK")
"###);
    assert_output(&out, r###"valid_signals_set_contents OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/wakeup_signal_tests__test_signum.py`.
#[test]
fn test_gen_behavior_std_libs_signal_wakeup_signal_tests__test_signum() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_signum"
# subject = "cpython.test_signal.WakeupSignalTests.test_signum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::WakeupSignalTests::test_signum
"""Auto-ported test: WakeupSignalTests::test_signum (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
def check_wakeup(test_body, *signals, ordered=True):
    code = 'if 1:\n        import _testcapi\n        import os\n        import signal\n        import struct\n\n        signals = {!r}\n\n        def handler(signum, frame):\n            pass\n\n        def check_signum(signals):\n            data = os.read(read, len(signals)+1)\n            raised = struct.unpack(\'%uB\' % len(data), data)\n            if not {!r}:\n                raised = set(raised)\n                signals = set(signals)\n            if raised != signals:\n                raise Exception("%r != %r" % (raised, signals))\n\n        {}\n\n        signal.signal(signal.SIGALRM, handler)\n        read, write = os.pipe()\n        os.set_blocking(write, False)\n        signal.set_wakeup_fd(write)\n\n        test()\n        check_signum(signals)\n\n        os.close(read)\n        os.close(write)\n        '.format(tuple(map(int, signals)), ordered, test_body)
    assert_python_ok('-c', code)
check_wakeup('def test():\n            signal.signal(signal.SIGUSR1, handler)\n            signal.raise_signal(signal.SIGUSR1)\n            signal.raise_signal(signal.SIGALRM)\n        ', signal.SIGUSR1, signal.SIGALRM)
print("WakeupSignalTests::test_signum: ok")
"###);
    assert_output(&out, r###"WakeupSignalTests::test_signum: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/wakeup_signal_tests__test_wakeup_fd_during.py`.
#[test]
fn test_gen_behavior_std_libs_signal_wakeup_signal_tests__test_wakeup_fd_during() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_wakeup_fd_during"
# subject = "cpython.test_signal.WakeupSignalTests.test_wakeup_fd_during"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::WakeupSignalTests::test_wakeup_fd_during
"""Auto-ported test: WakeupSignalTests::test_wakeup_fd_during (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
def check_wakeup(test_body, *signals, ordered=True):
    code = 'if 1:\n        import _testcapi\n        import os\n        import signal\n        import struct\n\n        signals = {!r}\n\n        def handler(signum, frame):\n            pass\n\n        def check_signum(signals):\n            data = os.read(read, len(signals)+1)\n            raised = struct.unpack(\'%uB\' % len(data), data)\n            if not {!r}:\n                raised = set(raised)\n                signals = set(signals)\n            if raised != signals:\n                raise Exception("%r != %r" % (raised, signals))\n\n        {}\n\n        signal.signal(signal.SIGALRM, handler)\n        read, write = os.pipe()\n        os.set_blocking(write, False)\n        signal.set_wakeup_fd(write)\n\n        test()\n        check_signum(signals)\n\n        os.close(read)\n        os.close(write)\n        '.format(tuple(map(int, signals)), ordered, test_body)
    assert_python_ok('-c', code)
check_wakeup('def test():\n            import select\n            import time\n\n            TIMEOUT_FULL = 10\n            TIMEOUT_HALF = 5\n\n            class InterruptSelect(Exception):\n                pass\n\n            def handler(signum, frame):\n                raise InterruptSelect\n            signal.signal(signal.SIGALRM, handler)\n\n            signal.alarm(1)\n            before_time = time.monotonic()\n            # We attempt to get a signal during the select call\n            try:\n                select.select([read], [], [], TIMEOUT_FULL)\n            except InterruptSelect:\n                pass\n            else:\n                raise Exception("select() was not interrupted")\n            after_time = time.monotonic()\n            dt = after_time - before_time\n            if dt >= TIMEOUT_HALF:\n                raise Exception("%s >= %s" % (dt, TIMEOUT_HALF))\n        ', signal.SIGALRM)
print("WakeupSignalTests::test_wakeup_fd_during: ok")
"###);
    assert_output(&out, r###"WakeupSignalTests::test_wakeup_fd_during: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/signal/wakeup_signal_tests__test_wakeup_fd_early.py`.
#[test]
fn test_gen_behavior_std_libs_signal_wakeup_signal_tests__test_wakeup_fd_early() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "wakeup_signal_tests__test_wakeup_fd_early"
# subject = "cpython.test_signal.WakeupSignalTests.test_wakeup_fd_early"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_signal.py::WakeupSignalTests::test_wakeup_fd_early
"""Auto-ported test: WakeupSignalTests::test_wakeup_fd_early (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
def check_wakeup(test_body, *signals, ordered=True):
    code = 'if 1:\n        import _testcapi\n        import os\n        import signal\n        import struct\n\n        signals = {!r}\n\n        def handler(signum, frame):\n            pass\n\n        def check_signum(signals):\n            data = os.read(read, len(signals)+1)\n            raised = struct.unpack(\'%uB\' % len(data), data)\n            if not {!r}:\n                raised = set(raised)\n                signals = set(signals)\n            if raised != signals:\n                raise Exception("%r != %r" % (raised, signals))\n\n        {}\n\n        signal.signal(signal.SIGALRM, handler)\n        read, write = os.pipe()\n        os.set_blocking(write, False)\n        signal.set_wakeup_fd(write)\n\n        test()\n        check_signum(signals)\n\n        os.close(read)\n        os.close(write)\n        '.format(tuple(map(int, signals)), ordered, test_body)
    assert_python_ok('-c', code)
check_wakeup('def test():\n            import select\n            import time\n\n            TIMEOUT_FULL = 10\n            TIMEOUT_HALF = 5\n\n            class InterruptSelect(Exception):\n                pass\n\n            def handler(signum, frame):\n                raise InterruptSelect\n            signal.signal(signal.SIGALRM, handler)\n\n            signal.alarm(1)\n\n            # We attempt to get a signal during the sleep,\n            # before select is called\n            try:\n                select.select([], [], [], TIMEOUT_FULL)\n            except InterruptSelect:\n                pass\n            else:\n                raise Exception("select() was not interrupted")\n\n            before_time = time.monotonic()\n            select.select([read], [], [], TIMEOUT_FULL)\n            after_time = time.monotonic()\n            dt = after_time - before_time\n            if dt >= TIMEOUT_HALF:\n                raise Exception("%s >= %s" % (dt, TIMEOUT_HALF))\n        ', signal.SIGALRM)
print("WakeupSignalTests::test_wakeup_fd_early: ok")
"###);
    assert_output(&out, r###"WakeupSignalTests::test_wakeup_fd_early: ok
"###);
}
