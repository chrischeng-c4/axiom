# Phase 1.B baseline smoke fixture for `mamba pytest` (pytest layout).
#
# Two free-function tests at module scope — one passes, one fails. Running
# `mamba pytest <this-dir>` must:
#   * discover both functions (no `class TestCase` wrapper),
#   * emit one PASS and one FAIL record,
#   * exit non-zero (acceptance gate #3).
#
# This fixture has no third-party imports — it only exercises the runner's
# discovery + harness synthesis pipeline. The conformance suite for actual
# PyPI libraries lives in `../<lib>/vendor_tests/` and follows the same
# `def test_*()` shape.
#
# Note: predicates are written so mamba's static lowering cannot
# const-fold `assert False` at module load time (which would short-circuit
# the load-check before per-test fan-out can reach the harness). Using
# `len(s)` keeps the expression dynamic.
#
# `import sys` is required because mamba's load-check otherwise auto-runs
# every top-level `def` body during compilation when no imports anchor the
# module. Any pytest vendor file imports something, so this is a faithful
# baseline (not a workaround unique to the runner).
import sys


def test_passes():
    x = len("hi")
    assert x == 2


def test_fails():
    x = len("hi")
    assert x == 99


# Reference sys so the import is not elided.
_ = sys.platform
