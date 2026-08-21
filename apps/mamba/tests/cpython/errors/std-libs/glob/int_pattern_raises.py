# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "errors"
# case = "int_pattern_raises"
# subject = "glob.glob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: int_pattern_raises (errors)."""
import glob

_raised = False
try:
    glob.glob(123)
except TypeError:
    _raised = True
assert _raised, "int_pattern_raises: expected TypeError"
print("int_pattern_raises OK")
