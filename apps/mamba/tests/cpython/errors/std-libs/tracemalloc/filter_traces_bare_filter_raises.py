# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "filter_traces_bare_filter_raises"
# subject = "tracemalloc.Snapshot"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: filter_traces_bare_filter_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.Snapshot([(0, 10, (('a.py', 2),), 1)], 1).filter_traces(tracemalloc.Filter(False, 'a.py'))
except TypeError:
    _raised = True
assert _raised, "filter_traces_bare_filter_raises: expected TypeError"
print("filter_traces_bare_filter_raises OK")
