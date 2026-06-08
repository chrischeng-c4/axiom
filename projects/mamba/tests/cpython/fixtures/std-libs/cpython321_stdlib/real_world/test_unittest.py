# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_unittest"
# subject = "cpython321.test_unittest"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_unittest.py"
# status = "filled"
# ///
"""cpython321.test_unittest: execute CPython 3.12 seed test_unittest"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_unittest.py — #2700 CPython unittest seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_unittest/ (the
# upstream suite is ~10k lines exercising TestCase / TestSuite /
# TestLoader / TestResult / TextTestRunner / mock / discover / etc).
# Instead it is the *smallest* Mamba-authored seed distilled from the
# unittest module's TOP-LEVEL SURFACE *and* the TestCase positive-path
# assertion methods — the parts that work on both CPython 3.12 and
# the mamba runtime today, and which prove the runtime's unittest
# dispatcher actually reaches TestCase method calls.
#
# Why so small? Mamba's current unittest surface presents the
# load-bearing names that benches and seeds depend on:
#
#   - `unittest.TestCase` is bound and callable; instantiating a
#     subclass yields an object whose `assertEqual` / `assertTrue` /
#     `assertIn` / `assertRaises` are bound methods.
#   - `unittest.main` / `unittest.skip` / `unittest.skipIf` /
#     `unittest.skipUnless` / `unittest.expectedFailure` are bound.
#
# But mamba's unittest stub does NOT yet provide:
#
#   - `unittest.TestSuite`, `unittest.TestLoader`, `unittest.TestResult`,
#     `unittest.TextTestRunner` (all return False from hasattr).
#   - `t.setUp`, `t.tearDown`, `t.fail` on a TestCase instance.
#   - Failing TestCase assertions (`t.assertEqual(1, 2)`) raise as a
#     Rust panic, not a Python-catchable AssertionError. We exercise
#     only positive-path assertion calls here — the negative path is
#     a separate gap tracked by #2545.
#
# The richer surface (full dispatch via `unittest.main()`, TestSuite
# discovery, TestResult collection, mock library) all land as each
# gap closes. Until then the load-bearing claim of this seed is
# narrow: the *module* exists, the TestCase/main/skip-family names
# are bound, a TestCase subclass instance can call assertEqual /
# assertTrue / assertIn on its positive arguments without raising,
# and the runner sees those calls actually execute through to a
# `MAMBA_ASSERTION_PASS` marker emitted *after* the assertion calls.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: unittest N asserts` to stdout.

import unittest

_ledger: list[int] = []

# 1. Module identity: unittest's own __name__ must be "unittest".
assert unittest.__name__ == "unittest", "unittest.__name__ must be 'unittest'"
_ledger.append(1)

# 2. TestCase is bound and callable. CPython has TestCase as a class;
#    mamba's stub has it as a builder. Either way the public name
#    must be both present and callable.
assert hasattr(unittest, "TestCase"), "unittest must expose TestCase"
_ledger.append(1)
assert callable(unittest.TestCase), "unittest.TestCase must be callable"
_ledger.append(1)

# 3. unittest.main is bound and callable. (On mamba today main() is
#    a no-op shim — #2545 will replace it with real dispatch.)
assert hasattr(unittest, "main"), "unittest must expose main"
_ledger.append(1)
assert callable(unittest.main), "unittest.main must be callable"
_ledger.append(1)

# 4. Skip-family decorators are bound. These are what real test
#    suites use to mark expected-fail / platform-conditional cases.
#    A regression that strips them would break every downstream
#    suite that decorates tests.
assert hasattr(unittest, "skip"), "unittest must expose skip"
_ledger.append(1)
assert hasattr(unittest, "skipIf"), "unittest must expose skipIf"
_ledger.append(1)
assert hasattr(unittest, "skipUnless"), "unittest must expose skipUnless"
_ledger.append(1)
assert hasattr(unittest, "expectedFailure"), "unittest must expose expectedFailure"
_ledger.append(1)

# 5. TestCase subclass instantiation. This is what proves the runtime
#    actually reaches a TestCase method-dispatch path: subclass it,
#    instantiate it, and confirm the assertion methods are bound.
class _UnitSeed(unittest.TestCase):
    def test_marker(self):
        pass

_case = _UnitSeed("test_marker")
assert type(_case).__name__ == "_UnitSeed", "TestCase subclass type identity preserved"
_ledger.append(1)
assert hasattr(_case, "assertEqual"), "TestCase instance must expose assertEqual"
_ledger.append(1)
assert hasattr(_case, "assertTrue"), "TestCase instance must expose assertTrue"
_ledger.append(1)
assert hasattr(_case, "assertIn"), "TestCase instance must expose assertIn"
_ledger.append(1)

# 6. Positive-path assertion calls. The runtime's unittest dispatcher
#    must reach the TestCase method body and return without raising
#    when the assertion holds. (Negative-path failures raise a Rust
#    panic on mamba today, not a Python AssertionError, so we exercise
#    only the positive path here — that gap is tracked by #2545.)
_case.assertEqual(1, 1)
_ledger.append(1)
_case.assertEqual("hello", "hello")
_ledger.append(1)
_case.assertTrue(True)
_ledger.append(1)
_case.assertTrue(1)
_ledger.append(1)
_case.assertIn(2, [1, 2, 3])
_ledger.append(1)
_case.assertIn("a", "banana")
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass. Crucially this marker is
# emitted *after* the TestCase positive-path assertion calls, which
# means the runner's summary captures "unittest dispatch as executed".
print(f"MAMBA_ASSERTION_PASS: unittest {len(_ledger)} asserts")
