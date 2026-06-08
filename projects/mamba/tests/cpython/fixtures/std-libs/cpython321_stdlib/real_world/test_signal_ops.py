# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_signal_ops"
# subject = "cpython321.test_signal_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_signal_ops.py"
# status = "filled"
# ///
"""cpython321.test_signal_ops: execute CPython 3.12 seed test_signal_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `signal` POSIX signal constants.
# Surface: SIGHUP/SIGINT/SIGABRT/SIGKILL/SIGTERM hold their canonical
# POSIX numeric values. Only the cross-platform-stable signals are
# asserted; SIGUSR1/SIGUSR2 differ between Linux (10/12) and macOS
# (30/31) and are intentionally omitted.
# Companion to stub/test_signal.py — vendored unittest seed.
import signal
_ledger: list[int] = []
assert signal.SIGHUP == 1; _ledger.append(1)
assert signal.SIGINT == 2; _ledger.append(1)
assert signal.SIGABRT == 6; _ledger.append(1)
assert signal.SIGKILL == 9; _ledger.append(1)
assert signal.SIGTERM == 15; _ledger.append(1)
# Strict ordering — POSIX numeric layout invariant
assert signal.SIGHUP < signal.SIGINT; _ledger.append(1)
assert signal.SIGINT < signal.SIGABRT; _ledger.append(1)
assert signal.SIGABRT < signal.SIGKILL; _ledger.append(1)
assert signal.SIGKILL < signal.SIGTERM; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_signal_ops {sum(_ledger)} asserts")
