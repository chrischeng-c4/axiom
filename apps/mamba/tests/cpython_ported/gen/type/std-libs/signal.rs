use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/signal/alarm__seconds_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_alarm__seconds_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "alarm__seconds_as_int_wrong"
# subject = "signal.alarm(seconds: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.alarm(seconds: int); call it with the wrong type.

typeshed contract: seconds is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import alarm
try:
    alarm("not_an_int")  # seconds: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/default_int_handler__signalnum_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_default_int_handler__signalnum_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "default_int_handler__signalnum_as_int_wrong"
# subject = "signal.default_int_handler(signalnum: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.default_int_handler(signalnum: int); call it with the wrong type.

typeshed contract: signalnum is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import default_int_handler
try:
    default_int_handler("not_an_int", None)  # signalnum: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/getitimer__which_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_getitimer__which_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "getitimer__which_as_int_wrong"
# subject = "signal.getitimer(which: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.getitimer(which: int); call it with the wrong type.

typeshed contract: which is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import getitimer
try:
    getitimer("not_an_int")  # which: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/getsignal__signalnum_as__SIGNUM_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_getsignal__signalnum_as__SIGNUM_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "getsignal__signalnum_as__SIGNUM_wrong"
# subject = "signal.getsignal(signalnum: _SIGNUM)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.getsignal(signalnum: _SIGNUM); call it with the wrong type.

typeshed contract: signalnum is _SIGNUM. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import getsignal
try:
    getsignal(_W())  # signalnum: _SIGNUM <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/pidfd_send_signal__pidfd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_pidfd_send_signal__pidfd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "pidfd_send_signal__pidfd_as_int_wrong"
# subject = "signal.pidfd_send_signal(pidfd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.pidfd_send_signal(pidfd: int); call it with the wrong type.

typeshed contract: pidfd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import pidfd_send_signal
try:
    pidfd_send_signal("not_an_int", 0)  # pidfd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/pthread_kill__thread_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_pthread_kill__thread_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "pthread_kill__thread_id_as_int_wrong"
# subject = "signal.pthread_kill(thread_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.pthread_kill(thread_id: int); call it with the wrong type.

typeshed contract: thread_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import pthread_kill
try:
    pthread_kill("not_an_int", 0)  # thread_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/pthread_sigmask__how_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_pthread_sigmask__how_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "pthread_sigmask__how_as_int_wrong"
# subject = "signal.pthread_sigmask(how: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.pthread_sigmask(how: int); call it with the wrong type.

typeshed contract: how is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import pthread_sigmask
try:
    pthread_sigmask("not_an_int", None)  # how: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/raise_signal__signalnum_as__SIGNUM_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_raise_signal__signalnum_as__SIGNUM_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "raise_signal__signalnum_as__SIGNUM_wrong"
# subject = "signal.raise_signal(signalnum: _SIGNUM)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.raise_signal(signalnum: _SIGNUM); call it with the wrong type.

typeshed contract: signalnum is _SIGNUM. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import raise_signal
try:
    raise_signal(_W())  # signalnum: _SIGNUM <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/set_wakeup_fd__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_set_wakeup_fd__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "set_wakeup_fd__fd_as_int_wrong"
# subject = "signal.set_wakeup_fd(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.set_wakeup_fd(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import set_wakeup_fd
try:
    set_wakeup_fd("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/setitimer__which_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_setitimer__which_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "setitimer__which_as_int_wrong"
# subject = "signal.setitimer(which: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.setitimer(which: int); call it with the wrong type.

typeshed contract: which is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import setitimer
try:
    setitimer("not_an_int", 0.0)  # which: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/siginterrupt__signalnum_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_siginterrupt__signalnum_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "siginterrupt__signalnum_as_int_wrong"
# subject = "signal.siginterrupt(signalnum: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.siginterrupt(signalnum: int); call it with the wrong type.

typeshed contract: signalnum is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import siginterrupt
try:
    siginterrupt("not_an_int", True)  # signalnum: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/signal__signalnum_as__SIGNUM_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_signal__signalnum_as__SIGNUM_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "signal__signalnum_as__SIGNUM_wrong"
# subject = "signal.signal(signalnum: _SIGNUM)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.signal(signalnum: _SIGNUM); call it with the wrong type.

typeshed contract: signalnum is _SIGNUM. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import signal
try:
    signal(_W(), None)  # signalnum: _SIGNUM <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/sigtimedwait__sigset_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_sigtimedwait__sigset_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "sigtimedwait__sigset_as_Iterable_wrong"
# subject = "signal.sigtimedwait(sigset: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.sigtimedwait(sigset: Iterable); call it with the wrong type.

typeshed contract: sigset is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import sigtimedwait
try:
    sigtimedwait(_W(), 0.0)  # sigset: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/sigwait__sigset_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_sigwait__sigset_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "sigwait__sigset_as_Iterable_wrong"
# subject = "signal.sigwait(sigset: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.sigwait(sigset: Iterable); call it with the wrong type.

typeshed contract: sigset is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import sigwait
try:
    sigwait(_W())  # sigset: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/sigwaitinfo__sigset_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_sigwaitinfo__sigset_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "sigwaitinfo__sigset_as_Iterable_wrong"
# subject = "signal.sigwaitinfo(sigset: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.sigwaitinfo(sigset: Iterable); call it with the wrong type.

typeshed contract: sigset is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import sigwaitinfo
try:
    sigwaitinfo(_W())  # sigset: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/signal/strsignal__signalnum_as__SIGNUM_wrong.py`.
#[test]
fn test_gen_type_std_libs_signal_strsignal__signalnum_as__SIGNUM_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "strsignal__signalnum_as__SIGNUM_wrong"
# subject = "signal.strsignal(signalnum: _SIGNUM)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.strsignal(signalnum: _SIGNUM); call it with the wrong type.

typeshed contract: signalnum is _SIGNUM. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from signal import strsignal
try:
    strsignal(_W())  # signalnum: _SIGNUM <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
