# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "errors"
# case = "iglob_int_pattern_raises"
# subject = "glob.iglob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.iglob: iglob_int_pattern_raises (errors)."""
import glob

_raised = False
try:
    list(glob.iglob(123))
except TypeError:
    _raised = True
assert _raised, "iglob_int_pattern_raises: expected TypeError"
print("iglob_int_pattern_raises OK")
