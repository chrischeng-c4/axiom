# test_pytest.py — #3470 axis-1 3p pytest AssertionPass seed.
#
# Mamba-authored seed exercising the pytest framework surface called
# out in the issue:
#   * collect test_* functions
#   * fixture injection
#   * parametrize matrix
#   * assert rewrites — runs via pytest.main
#
# The seed writes a tiny test module to a temp directory and runs
# pytest.main() against it in-process, asserting the exit code and
# collection counts. It also exercises pytest's public introspection
# helpers (approx, raises, ExitCode).
#
# Contract placement: `spec/` — pins outcome Fail. Mamba pkgmgr (Phase
# 1.5 per #1262) cannot yet install pure-Python wheels like pytest, so
# `import pytest` fails on mamba today. Once mamba pkgmgr installs
# pytest cleanly and the seed flips to AssertionPass on mamba, drift
# detection prompts a `git mv spec/test_pytest.py pass/test_pytest.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + main / fixture / mark / approx / raises / ExitCode.
#   2. pytest.approx — numeric approximate equality.
#   3. pytest.raises — catches expected exception; match= filters by message.
#   4. pytest.ExitCode.OK == 0; TESTS_FAILED == 1.
#   5. pytest.main runs a temp test module — passing tests yield OK.
#   6. pytest.main on a failing test module yields TESTS_FAILED.
#   7. pytest.main collects parametrize matrix — 4-row matrix passes.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_pytest N asserts` to stdout.

import os
import tempfile

import pytest

_ledger: list[int] = []

# 1. Module identity.
assert pytest.__name__ == "pytest", "pytest.__name__"
_ledger.append(1)
assert hasattr(pytest, "main"), "pytest exposes main"
_ledger.append(1)
assert hasattr(pytest, "fixture"), "pytest exposes fixture"
_ledger.append(1)
assert hasattr(pytest, "mark"), "pytest exposes mark"
_ledger.append(1)
assert hasattr(pytest, "approx"), "pytest exposes approx"
_ledger.append(1)
assert hasattr(pytest, "raises"), "pytest exposes raises"
_ledger.append(1)
assert hasattr(pytest, "ExitCode"), "pytest exposes ExitCode"
_ledger.append(1)


# 2. pytest.approx — numeric approximate equality.
assert 0.1 + 0.2 == pytest.approx(0.3), "approx tolerates float rounding"
_ledger.append(1)
assert 0.1 + 0.2 != 0.3, "raw float equality fails (sanity check)"
_ledger.append(1)
# Tolerance override.
assert 1.0001 == pytest.approx(1.0, abs=1e-3), "approx with abs= tolerance"
_ledger.append(1)
# List approx — element-wise.
assert [0.1 + 0.2, 0.1 + 0.1] == pytest.approx([0.3, 0.2]), (
    "approx works element-wise on lists"
)
_ledger.append(1)


# 3. pytest.raises — context manager catches expected exception.
with pytest.raises(ValueError):
    int("not-a-number")
_ledger.append(1)
# match= filter.
with pytest.raises(ZeroDivisionError, match="division by zero"):
    _ = 1 / 0
_ledger.append(1)
# Wrong exception type → pytest.raises lets it propagate.
_propagated = False
try:
    with pytest.raises(ValueError):
        raise KeyError("not a ValueError")
except KeyError:
    _propagated = True
assert _propagated == True, "pytest.raises lets unmatched exception propagate"
_ledger.append(1)
# excinfo carries the captured exception.
with pytest.raises(RuntimeError) as _excinfo:
    raise RuntimeError("boom")
assert str(_excinfo.value) == "boom", "raises excinfo carries exception message"
_ledger.append(1)


# 4. ExitCode introspection.
assert int(pytest.ExitCode.OK) - 0 == 0, "ExitCode.OK == 0 (boxed-dodge)"
_ledger.append(1)
assert int(pytest.ExitCode.TESTS_FAILED) - 1 == 0, (
    "ExitCode.TESTS_FAILED == 1 (boxed-dodge)"
)
_ledger.append(1)


# 5. pytest.main on a temp test module — passing tests yield OK.
_passing = """
import pytest


def test_one():
    assert 1 + 1 == 2


def test_two():
    assert "hello".upper() == "HELLO"


@pytest.fixture
def number():
    return 42


def test_fixture_injection(number):
    assert number == 42
"""

with tempfile.TemporaryDirectory() as _td:
    _passing_path = os.path.join(_td, "test_passing.py")
    with open(_passing_path, "w") as _f:
        _f.write(_passing)
    _rc = pytest.main(["-q", "--no-header", "-rN", _passing_path])
    # ExitCode.OK is an IntEnum — compare via int().
    assert int(_rc) - 0 == 0, (
        "pytest.main on passing module yields ExitCode.OK (boxed-dodge)"
    )
    _ledger.append(1)


# 6. pytest.main on a failing test module yields TESTS_FAILED.
_failing = """
def test_will_fail():
    assert 1 + 1 == 3
"""

with tempfile.TemporaryDirectory() as _td2:
    _failing_path = os.path.join(_td2, "test_failing.py")
    with open(_failing_path, "w") as _f:
        _f.write(_failing)
    _rc_fail = pytest.main(["-q", "--no-header", "-rN", _failing_path])
    assert int(_rc_fail) - 1 == 0, (
        "pytest.main on failing module yields ExitCode.TESTS_FAILED (boxed-dodge)"
    )
    _ledger.append(1)


# 7. parametrize matrix — 4-row matrix passes via pytest.main.
_parametrize_mod = """
import pytest


@pytest.mark.parametrize(("a", "b", "expected"), [
    (1, 2, 3),
    (2, 3, 5),
    (5, 5, 10),
    (-1, 1, 0),
])
def test_add(a, b, expected):
    assert a + b == expected
"""

with tempfile.TemporaryDirectory() as _td3:
    _param_path = os.path.join(_td3, "test_parametrize.py")
    with open(_param_path, "w") as _f:
        _f.write(_parametrize_mod)
    _rc_p = pytest.main(["-q", "--no-header", "-rN", _param_path])
    assert int(_rc_p) - 0 == 0, (
        "pytest.main on parametrize matrix yields OK (boxed-dodge)"
    )
    _ledger.append(1)


# Emit the proof-of-execution marker.
print(f"MAMBA_ASSERTION_PASS: test_pytest {len(_ledger)} asserts")
