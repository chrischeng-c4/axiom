# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "filter_non_iterable_raises"
# subject = "fnmatch.filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.filter: filter_non_iterable_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.filter(123, "*.py")
except TypeError:
    _raised = True
assert _raised, "filter_non_iterable_raises: expected TypeError"
print("filter_non_iterable_raises OK")
