# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "filter_bad_keyword_raises"
# subject = "tracemalloc.Filter"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: filter_bad_keyword_raises (errors)."""
import tracemalloc

_raised = False
try:
    tracemalloc.Filter(True, no_such_kwarg='x')
except TypeError:
    _raised = True
assert _raised, "filter_bad_keyword_raises: expected TypeError"
print("filter_bad_keyword_raises OK")
