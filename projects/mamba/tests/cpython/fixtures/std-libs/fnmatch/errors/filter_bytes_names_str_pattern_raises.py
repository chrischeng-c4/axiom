# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "filter_bytes_names_str_pattern_raises"
# subject = "fnmatch.filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.filter: filter_bytes_names_str_pattern_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.filter([b"test"], "*")
except TypeError:
    _raised = True
assert _raised, "filter_bytes_names_str_pattern_raises: expected TypeError"
print("filter_bytes_names_str_pattern_raises OK")
