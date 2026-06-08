# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "errors"
# case = "filter_field_readonly_raises"
# subject = "tracemalloc.Filter"
# kind = "mechanical"
# xfail = "mamba does not implement the tracemalloc.Filter class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Filter: filter_field_readonly_raises (errors)."""
import tracemalloc
_f = tracemalloc.Filter(True, 'abc')

_raised = False
try:
    setattr(_f, 'filename_pattern', 'x')
except AttributeError:
    _raised = True
assert _raised, "filter_field_readonly_raises: expected AttributeError"
print("filter_field_readonly_raises OK")
