# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "errors"
# case = "s_imode_non_int_raises"
# subject = "stat.S_IMODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IMODE: s_imode_non_int_raises (errors)."""
import stat

_raised = False
try:
    stat.S_IMODE("not_int")
except TypeError:
    _raised = True
assert _raised, "s_imode_non_int_raises: expected TypeError"
print("s_imode_non_int_raises OK")
