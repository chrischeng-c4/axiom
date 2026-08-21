# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "start_negative_nframe_raises"
# subject = "tracemalloc.start"
# kind = "mechanical"
# xfail = "mamba tracemalloc.start is a no-op shim; negative nframe is not validated, so no ValueError is raised (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.start: start_negative_nframe_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.start(-1)
except ValueError:
    _raised = True
assert _raised, "start_negative_nframe_raises: expected ValueError"
print("start_negative_nframe_raises OK")
