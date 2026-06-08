# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_logging"
# subject = "cpython321.test_logging"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_logging.py"
# status = "filled"
# ///
"""cpython321.test_logging: execute CPython 3.12 seed test_logging"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_logging.py — #2699 CPython logging seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_logging.py
# (the upstream file is ~5000 lines exercising getLogger / setLevel /
# Handler / Formatter / StreamHandler / FileHandler / Logger hierarchy
# / LogRecord / propagation / filter / etc). Instead it is the
# *smallest* Mamba-authored seed distilled from the logging module's
# TOP-LEVEL CONSTANTS surface — the parts that work on both CPython
# 3.12 and the mamba runtime today.
#
# Why so small? Mamba's current logging is a stub: the module imports
# cleanly and the level constants (DEBUG/INFO/WARNING/ERROR/CRITICAL)
# have the correct numeric values, but
#
#   - `logging.getLogger("test")` returns a *dict*, not a real Logger
#     (so `.setLevel(...)` raises AttributeError: 'dict' object has
#     no attribute 'setLevel').
#   - `logging.Logger`, `logging.Handler`, `logging.Formatter`,
#     `logging.StreamHandler`, `logging.LogRecord`, `logging.getLevelName`
#     all return False from hasattr() on mamba.
#   - `logging.NOTSET` is None on mamba (should be 0 on CPython).
#
# The richer surface (getLogger().setLevel/info/warning, Logger
# hierarchy with dotted names, Handler/Formatter, basicConfig with
# level=, LogRecord shape) all land as each gap closes. Until then
# the load-bearing claim of this seed is narrow: the *module* exists,
# the five RFC-3164-style level constants have their canonical values
# and ordering, and the `getLogger` / `basicConfig` top-level names
# are bound. That is genuinely deterministic on both runtimes and a
# regression in either direction would catch a real bootstrap break.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: logging N asserts` to stdout.

import logging

_ledger: list[int] = []

# 1. Module identity: logging's own __name__ must be "logging".
assert logging.__name__ == "logging", "logging.__name__ must be 'logging'"
_ledger.append(1)

# 2. Canonical level constants. The numeric values are part of the
#    public contract of the stdlib logging module (RFC-3164-style
#    severity codes). A regression here means severity comparisons
#    in any downstream user of logging would silently invert.
assert logging.DEBUG == 10, "logging.DEBUG must be 10"
_ledger.append(1)
assert logging.INFO == 20, "logging.INFO must be 20"
_ledger.append(1)
assert logging.WARNING == 30, "logging.WARNING must be 30"
_ledger.append(1)
assert logging.ERROR == 40, "logging.ERROR must be 40"
_ledger.append(1)
assert logging.CRITICAL == 50, "logging.CRITICAL must be 50"
_ledger.append(1)

# 3. Level ordering. The strict-less-than chain is what downstream
#    `Logger.isEnabledFor(level)` comparisons rely on. Pinning this
#    as its own assert (rather than only the literal values) catches
#    a regression that *renames* the constants but breaks the chain.
assert logging.DEBUG < logging.INFO, "DEBUG < INFO"
_ledger.append(1)
assert logging.INFO < logging.WARNING, "INFO < WARNING"
_ledger.append(1)
assert logging.WARNING < logging.ERROR, "WARNING < ERROR"
_ledger.append(1)
assert logging.ERROR < logging.CRITICAL, "ERROR < CRITICAL"
_ledger.append(1)

# 4. getLogger is bound to a callable on the module. Catches a
#    regression where the import returns an empty stub. (The
#    *return value* of getLogger("name") is a dict on mamba today,
#    not a real Logger, so this seed does not exercise it.)
assert hasattr(logging, "getLogger"), "logging must expose getLogger"
_ledger.append(1)
assert callable(logging.getLogger), "logging.getLogger must be callable"
_ledger.append(1)

# 5. basicConfig is bound. CPython's logging exposes this as the
#    one-call configurator; mamba's stub has the name bound (though
#    invoking it is a no-op on mamba). Pinning the name catches a
#    regression where the stub is stripped further.
assert hasattr(logging, "basicConfig"), "logging must expose basicConfig"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: logging {len(_ledger)} asserts")
