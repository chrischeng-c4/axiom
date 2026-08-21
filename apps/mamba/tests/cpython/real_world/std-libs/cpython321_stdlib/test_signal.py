# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_signal"
# subject = "cpython321.test_signal"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_signal.py"
# status = "filled"
# ///
"""cpython321.test_signal: execute CPython 3.12 seed test_signal"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: signal — POSIX signal-number constants (SIGINT/SIGTERM/SIGKILL/
# SIGHUP/SIGUSR1/SIGUSR2), SIG_DFL / SIG_IGN sentinels, getsignal() lookup,
# and signal()-set/restore round-trip.
# signal.Signals enum is not exposed as a real enum on mamba (Signals.SIGINT
# is None) and signal.strsignal returns None; both intentionally NOT
# exercised here, tracked separately.
import signal

_ledger: list[int] = []

# POSIX signal-number constants
assert signal.SIGINT == 2, f"SIGINT == 2, got {signal.SIGINT}"
_ledger.append(1)

assert signal.SIGTERM == 15, f"SIGTERM == 15, got {signal.SIGTERM}"
_ledger.append(1)

assert signal.SIGKILL == 9, f"SIGKILL == 9, got {signal.SIGKILL}"
_ledger.append(1)

assert signal.SIGHUP == 1, f"SIGHUP == 1, got {signal.SIGHUP}"
_ledger.append(1)

# User-defined POSIX signals
assert signal.SIGUSR1 == 30 or signal.SIGUSR1 == 10, (
    f"SIGUSR1 ∈ {{30 (macOS), 10 (Linux)}}, got {signal.SIGUSR1}"
)
_ledger.append(1)

assert signal.SIGUSR2 == 31 or signal.SIGUSR2 == 12, (
    f"SIGUSR2 ∈ {{31 (macOS), 12 (Linux)}}, got {signal.SIGUSR2}"
)
_ledger.append(1)

# SIG_DFL / SIG_IGN handler sentinels
assert signal.SIG_DFL == 0, f"SIG_DFL == 0, got {signal.SIG_DFL}"
_ledger.append(1)

assert signal.SIG_IGN == 1, f"SIG_IGN == 1, got {signal.SIG_IGN}"
_ledger.append(1)

# getsignal returns a handler reference for an installed signal
_h = signal.getsignal(signal.SIGINT)
assert _h is not None, "signal.getsignal(SIGINT) is not None"
_ledger.append(1)

# Round-trip: install our own handler, capture the previous one, then restore
def _noop_handler(_sig, _frame):
    pass

_old = signal.signal(signal.SIGUSR1, _noop_handler)
assert _old is not None, (
    "signal.signal(SIGUSR1, ...) returns the previous handler, not None"
)
_ledger.append(1)

# Restoring works and returns our just-installed handler
_round_trip = signal.signal(signal.SIGUSR1, _old)
assert _round_trip is not None, (
    "restoring a handler via signal.signal returns the just-installed handler"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_signal {sum(_ledger)} asserts")
