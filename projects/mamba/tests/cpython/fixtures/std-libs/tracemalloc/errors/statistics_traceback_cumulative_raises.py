# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "statistics_traceback_cumulative_raises"
# subject = "tracemalloc.Snapshot"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics_traceback_cumulative_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.Snapshot([(0, 10, (('a.py', 2),), 1)], 1).statistics('traceback', cumulative=True)
except ValueError:
    _raised = True
assert _raised, "statistics_traceback_cumulative_raises: expected ValueError"
print("statistics_traceback_cumulative_raises OK")
