# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "errors"
# case = "bad_lineno_raises"
# subject = "warnings.warn_explicit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: bad_lineno_raises (errors)."""
import warnings

_raised = False
try:
    warnings.warn_explicit("hi", UserWarning, "file", "not_an_int")
except TypeError:
    _raised = True
assert _raised, "bad_lineno_raises: expected TypeError"
print("bad_lineno_raises OK")
