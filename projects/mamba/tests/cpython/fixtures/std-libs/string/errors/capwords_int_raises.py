# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "errors"
# case = "capwords_int_raises"
# subject = "string.capwords"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.capwords: capwords_int_raises (errors)."""
import string

_raised = False
try:
    string.capwords(123)
except AttributeError:
    _raised = True
assert _raised, "capwords_int_raises: expected AttributeError"
print("capwords_int_raises OK")
