# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "errors"
# case = "translate_int_raises"
# subject = "fnmatch.translate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.translate: translate_int_raises (errors)."""
import fnmatch

_raised = False
try:
    fnmatch.translate(123)
except TypeError:
    _raised = True
assert _raised, "translate_int_raises: expected TypeError"
print("translate_int_raises OK")
