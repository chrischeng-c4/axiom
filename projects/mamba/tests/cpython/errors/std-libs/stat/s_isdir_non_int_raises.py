# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "errors"
# case = "s_isdir_non_int_raises"
# subject = "stat.S_ISDIR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_stat.py"
# status = "filled"
# ///
"""stat.S_ISDIR: s_isdir_non_int_raises (errors)."""
import stat

_raised = False
try:
    stat.S_ISDIR("not_int")
except TypeError:
    _raised = True
assert _raised, "s_isdir_non_int_raises: expected TypeError"
print("s_isdir_non_int_raises OK")
