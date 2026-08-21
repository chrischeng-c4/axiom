# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "fnmatchcase_bytes_name_str_pattern_raises"
# subject = "fnmatch.fnmatchcase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatchcase: fnmatchcase_bytes_name_str_pattern_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.fnmatchcase(b"test", "*")
except TypeError:
    _raised = True
assert _raised, "fnmatchcase_bytes_name_str_pattern_raises: expected TypeError"
print("fnmatchcase_bytes_name_str_pattern_raises OK")
