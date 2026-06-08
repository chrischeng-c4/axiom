# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "errors"
# case = "s_ifmt_non_int_raises"
# subject = "stat.S_IFMT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""stat.S_IFMT: s_ifmt_non_int_raises (errors)."""
import stat

_raised = False
try:
    stat.S_IFMT(None)
except TypeError:
    _raised = True
assert _raised, "s_ifmt_non_int_raises: expected TypeError"
print("s_ifmt_non_int_raises OK")
