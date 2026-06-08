# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "bad_category_raises"
# subject = "warnings.warn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: bad_category_raises (errors)."""
import warnings

_raised = False
try:
    warnings.warn("hi", category=int)
except TypeError:
    _raised = True
assert _raised, "bad_category_raises: expected TypeError"
print("bad_category_raises OK")
