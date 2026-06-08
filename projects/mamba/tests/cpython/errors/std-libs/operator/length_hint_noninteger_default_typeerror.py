# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "length_hint_noninteger_default_typeerror"
# subject = "operator.length_hint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: length_hint_noninteger_default_typeerror (errors)."""
import operator

_raised = False
try:
    operator.length_hint(object(), "abc")
except TypeError:
    _raised = True
assert _raised, "length_hint_noninteger_default_typeerror: expected TypeError"
print("length_hint_noninteger_default_typeerror OK")
