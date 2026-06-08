# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "errors"
# case = "none_pattern_raises"
# subject = "glob.glob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: none_pattern_raises (errors)."""
import glob

_raised = False
try:
    glob.glob(None)
except TypeError:
    _raised = True
assert _raised, "none_pattern_raises: expected TypeError"
print("none_pattern_raises OK")
