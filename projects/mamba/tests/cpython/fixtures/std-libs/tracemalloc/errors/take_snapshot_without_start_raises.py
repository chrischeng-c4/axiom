# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "take_snapshot_without_start_raises"
# subject = "tracemalloc.take_snapshot"
# kind = "mechanical"
# xfail = "mamba tracemalloc is a GC-counter shim; take_snapshot returns a stub instead of raising RuntimeError when not tracing (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.take_snapshot: take_snapshot_without_start_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.take_snapshot()
except RuntimeError:
    _raised = True
assert _raised, "take_snapshot_without_start_raises: expected RuntimeError"
print("take_snapshot_without_start_raises OK")
