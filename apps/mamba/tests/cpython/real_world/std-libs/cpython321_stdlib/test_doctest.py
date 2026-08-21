# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_doctest"
# subject = "cpython321.test_doctest"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_doctest.py"
# status = "filled"
# ///
"""cpython321.test_doctest: execute CPython 3.12 seed test_doctest"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_doctest.py — #3451 axis-1 stdlib doctest AssertionPass seed.
#
# Mamba-authored seed exercising the `doctest` module surface called
# out in the issue:
#   testmod / testfile, DocTestRunner, ELLIPSIS, NORMALIZE_WHITESPACE,
#   expected output match.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. ELLIPSIS + NORMALIZE_WHITESPACE option-flag sentinels exposed
#      and distinct.
#   3. DocTestParser parses inline doctests from a string source —
#      returns a list of Example objects with `.source` and `.want`.
#   4. DocTestRunner.run() on a passing doctest yields zero failures
#      and the expected attempt count.
#   5. ELLIPSIS option matches `...` against arbitrary substrings.
#   6. NORMALIZE_WHITESPACE option matches multi-line whitespace.
#   7. testfile() against a tempfile doctest returns (failed, attempted)
#      with failed == 0.
#
# Boxed-int dodge (subtraction-against-zero) applied for count checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: doctest N asserts` to stdout.

import doctest
import os
import tempfile

_ledger: list[int] = []

# 1. Module identity + public surface.
assert doctest.__name__ == "doctest", "doctest.__name__"
_ledger.append(1)
assert hasattr(doctest, "testmod"), "exposes testmod"
_ledger.append(1)
assert hasattr(doctest, "testfile"), "exposes testfile"
_ledger.append(1)
assert hasattr(doctest, "DocTestRunner"), "exposes DocTestRunner"
_ledger.append(1)
assert hasattr(doctest, "DocTestParser"), "exposes DocTestParser"
_ledger.append(1)
assert hasattr(doctest, "Example"), "exposes Example"
_ledger.append(1)
assert hasattr(doctest, "ELLIPSIS"), "exposes ELLIPSIS option flag"
_ledger.append(1)
assert hasattr(doctest, "NORMALIZE_WHITESPACE"), "exposes NORMALIZE_WHITESPACE option flag"
_ledger.append(1)

# 2. Option-flag sentinels.
assert isinstance(doctest.ELLIPSIS, int), "ELLIPSIS option flag is an int bitmask"
_ledger.append(1)
assert isinstance(doctest.NORMALIZE_WHITESPACE, int), "NORMALIZE_WHITESPACE option flag is an int bitmask"
_ledger.append(1)
assert doctest.ELLIPSIS != doctest.NORMALIZE_WHITESPACE, "ELLIPSIS != NORMALIZE_WHITESPACE"
_ledger.append(1)

# 3. DocTestParser — parse inline doctests.
_parser = doctest.DocTestParser()
_doctest_src = (
    ">>> 1 + 1\n"
    "2\n"
    ">>> 'mamba'.upper()\n"
    "'MAMBA'\n"
)
_examples = _parser.get_examples(_doctest_src)
assert isinstance(_examples, list), "DocTestParser.get_examples returns a list"
_ledger.append(1)
# Boxed-int dodge for length check.
assert len(_examples) - 2 == 0, "parser found exactly 2 examples"
_ledger.append(1)
_ex0 = _examples[0]
assert isinstance(_ex0, doctest.Example), "examples are Example instances"
_ledger.append(1)
assert _ex0.source.rstrip("\n") == "1 + 1", "Example.source matches '1 + 1'"
_ledger.append(1)
assert _ex0.want.rstrip("\n") == "2", "Example.want matches '2'"
_ledger.append(1)
_ex1 = _examples[1]
assert _ex1.source.rstrip("\n") == "'mamba'.upper()", "second Example.source matches"
_ledger.append(1)
assert _ex1.want.rstrip("\n") == "'MAMBA'", "second Example.want matches"
_ledger.append(1)

# 4. DocTestRunner — run a passing doctest.
_dt = _parser.get_doctest(_doctest_src, {}, "inline", "<inline>", 0)
_runner = doctest.DocTestRunner(verbose=False)
_results = _runner.run(_dt)
# In CPython 3.4+, run() returns a TestResults(failed, attempted) namedtuple.
assert _results.failed == 0, "passing doctest reports 0 failures"
_ledger.append(1)
assert _results.attempted - 2 == 0, "runner attempted 2 examples"
_ledger.append(1)

# 5. ELLIPSIS option — `...` matches arbitrary substring in expected output.
_ellipsis_src = (
    ">>> 'hello mamba world'\n"
    "'hello ... world'\n"
)
_dt_e = _parser.get_doctest(_ellipsis_src, {}, "ell", "<ell>", 0)
_runner_e = doctest.DocTestRunner(verbose=False, optionflags=doctest.ELLIPSIS)
_results_e = _runner_e.run(_dt_e)
assert _results_e.failed == 0, "ELLIPSIS matches '...' against substring"
_ledger.append(1)
assert _results_e.attempted - 1 == 0, "ELLIPSIS test attempted 1 example"
_ledger.append(1)

# Without the ELLIPSIS flag, the same doctest must FAIL.
_runner_no_e = doctest.DocTestRunner(verbose=False)
_dt_e2 = _parser.get_doctest(_ellipsis_src, {}, "ell2", "<ell2>", 0)
_results_no_e = _runner_no_e.run(_dt_e2)
assert _results_no_e.failed == 1, "without ELLIPSIS, '...' is literal — mismatch"
_ledger.append(1)

# 6. NORMALIZE_WHITESPACE — collapses runs of whitespace.
_nw_src = (
    ">>> ' '.join(['a', 'b', 'c'])\n"
    "'a   b   c'\n"
)
_dt_nw = _parser.get_doctest(_nw_src, {}, "nw", "<nw>", 0)
_runner_nw = doctest.DocTestRunner(
    verbose=False, optionflags=doctest.NORMALIZE_WHITESPACE
)
_results_nw = _runner_nw.run(_dt_nw)
assert _results_nw.failed == 0, "NORMALIZE_WHITESPACE collapses ws differences"
_ledger.append(1)

# 7. testfile — write a tempfile and exercise the public surface.
_tmpdir = tempfile.mkdtemp()
_path = os.path.join(_tmpdir, "seed_doctest.txt")
_fh = open(_path, "w")
_fh.write(
    "Sample doctest file.\n"
    "\n"
    "    >>> 2 * 21\n"
    "    42\n"
    "    >>> 'mamba'[::-1]\n"
    "    'abmam'\n"
)
_fh.close()
_tf_result = doctest.testfile(
    _path,
    module_relative=False,
    verbose=False,
)
# testfile returns TestResults(failed, attempted).
assert _tf_result.failed == 0, "testfile against passing fixture has 0 failures"
_ledger.append(1)
assert _tf_result.attempted - 2 == 0, "testfile against passing fixture attempted 2"
_ledger.append(1)

# Cleanup the tempdir; not load-bearing for the assertion count.
os.remove(_path)
os.rmdir(_tmpdir)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: doctest {len(_ledger)} asserts")
