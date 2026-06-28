# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_signal_constants_ops"
# subject = "cpython321.test_signal_constants_ops"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_signal_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_signal_constants_ops: execute CPython 3.12 seed test_signal_constants_ops"""
# Operational AssertionPass seed for the `signal` module — the
# stdlib wrapper over POSIX signal numbers (SIGINT / SIGTERM /
# SIGKILL / etc.) used by subprocess management, server shutdown
# (SIGTERM trap), signal-based interrupts, and timer scaffolding
# (SIGALRM). Surface focuses on the canonical POSIX signal-number
# constants — these are part of the OS ABI and stable across mamba
# / CPython on the same platform (macOS arm64). Both runtimes expose
# the integer value of each signal directly through `signal.Signals`
# IntEnum members. `isinstance(SIGINT, int)` is True in both runtimes
# because `Signals` is an `IntEnum` subclass — so equality with the
# bare int still works.
#
# Surface (all canonical POSIX signal numbers, per macOS / BSD ABI):
#   • signal.SIGHUP == 1;  signal.SIGINT == 2;  signal.SIGQUIT == 3;
#   • signal.SIGILL == 4;  signal.SIGTRAP == 5; signal.SIGABRT == 6;
#   • signal.SIGFPE == 8;  signal.SIGKILL == 9; signal.SIGSEGV == 11;
#   • signal.SIGPIPE == 13; signal.SIGALRM == 14; signal.SIGTERM == 15;
#   • signal.SIGSTOP == 17; signal.SIGTSTP == 18; signal.SIGCONT == 19;
#   • signal.SIGCHLD == 20; signal.SIGTTIN == 21; signal.SIGTTOU == 22;
#   • signal.SIGUSR1 == 30; signal.SIGUSR2 == 31;
#   • signal.SIG_DFL == 0; signal.SIG_IGN == 1;
#   • signal.NSIG == 32 (POSIX limit on signal count).
import signal
_ledger: list[int] = []

# Canonical POSIX signal numbers (macOS arm64 + Linux agree on these)
assert signal.SIGHUP == 1; _ledger.append(1)
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGQUIT == 3; _ledger.append(1)
assert signal.SIGILL == 4; _ledger.append(1)
assert signal.SIGTRAP == 5; _ledger.append(1)
assert signal.SIGABRT == 6; _ledger.append(1)
assert signal.SIGFPE == 8; _ledger.append(1)
assert signal.SIGKILL == 9; _ledger.append(1)
assert signal.SIGSEGV == 11; _ledger.append(1)
assert signal.SIGPIPE == 13; _ledger.append(1)
assert signal.SIGALRM == 14; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)
assert signal.SIGSTOP == 17; _ledger.append(1)
assert signal.SIGTSTP == 18; _ledger.append(1)
assert signal.SIGCONT == 19; _ledger.append(1)
assert signal.SIGCHLD == 20; _ledger.append(1)
assert signal.SIGTTIN == 21; _ledger.append(1)
assert signal.SIGTTOU == 22; _ledger.append(1)
assert signal.SIGUSR1 == 30; _ledger.append(1)
assert signal.SIGUSR2 == 31; _ledger.append(1)

# Disposition constants
assert signal.SIG_DFL == 0; _ledger.append(1)
assert signal.SIG_IGN == 1; _ledger.append(1)

# Signal count cap
assert signal.NSIG == 32; _ledger.append(1)

# Each signal is int-typed (or IntEnum-typed → int-compatible)
assert isinstance(signal.SIGINT, int); _ledger.append(1)
assert isinstance(signal.SIGTERM, int); _ledger.append(1)
assert isinstance(signal.SIGKILL, int); _ledger.append(1)
assert isinstance(signal.SIGHUP, int); _ledger.append(1)
assert isinstance(signal.SIGUSR1, int); _ledger.append(1)
assert isinstance(signal.SIGUSR2, int); _ledger.append(1)
assert isinstance(signal.SIG_DFL, int); _ledger.append(1)
assert isinstance(signal.SIG_IGN, int); _ledger.append(1)

# Cross-signal numeric comparisons — each signal is unique
_signals_seen = [signal.SIGHUP, signal.SIGINT, signal.SIGQUIT,
                 signal.SIGILL, signal.SIGTRAP, signal.SIGABRT,
                 signal.SIGFPE, signal.SIGKILL, signal.SIGSEGV,
                 signal.SIGPIPE, signal.SIGALRM, signal.SIGTERM,
                 signal.SIGSTOP, signal.SIGTSTP, signal.SIGCONT,
                 signal.SIGCHLD, signal.SIGTTIN, signal.SIGTTOU,
                 signal.SIGUSR1, signal.SIGUSR2]
assert len(_signals_seen) == len(set(_signals_seen)); _ledger.append(1)

# Each signal value is < NSIG
for _s in _signals_seen:
    assert 0 < _s < signal.NSIG; _ledger.append(1)

# SIGKILL == 9 is the canonical "uncatchable" signal
assert signal.SIGKILL == 9; _ledger.append(1)
# SIGSTOP == 17 is the canonical "uncatchable stop" signal
assert signal.SIGSTOP == 17; _ledger.append(1)
# SIGUSR1 and SIGUSR2 are user-defined slots (mamba+CPython agree)
assert signal.SIGUSR1 == 30; _ledger.append(1)
assert signal.SIGUSR2 == 31; _ledger.append(1)
assert signal.SIGUSR1 != signal.SIGUSR2; _ledger.append(1)

# SIG_DFL != SIG_IGN — they're distinct dispositions
assert signal.SIG_DFL != signal.SIG_IGN; _ledger.append(1)

# Repeatable — calling twice returns same value (attribute lookup
# is pure / deterministic)
assert signal.SIGINT == signal.SIGINT; _ledger.append(1)
assert signal.SIGTERM == signal.SIGTERM; _ledger.append(1)
assert signal.SIG_DFL == signal.SIG_DFL; _ledger.append(1)

# getsignal is a callable (the module-level handler-introspection fn)
assert callable(signal.getsignal); _ledger.append(1)
# Signals attribute (enum class on CPython, lambda factory on mamba)
# is present in both runtimes
assert hasattr(signal, "Signals"); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_signal_constants_ops {sum(_ledger)} asserts")
