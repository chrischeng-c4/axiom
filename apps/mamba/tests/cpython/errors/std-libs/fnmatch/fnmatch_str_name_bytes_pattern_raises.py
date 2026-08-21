# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "fnmatch_str_name_bytes_pattern_raises"
# subject = "fnmatch.fnmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatch: fnmatch_str_name_bytes_pattern_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.fnmatch("test", b"*")
except TypeError:
    _raised = True
assert _raised, "fnmatch_str_name_bytes_pattern_raises: expected TypeError"
print("fnmatch_str_name_bytes_pattern_raises OK")
