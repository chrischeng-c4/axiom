# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_missing_method_attributeerror"
# subject = "operator.methodcaller"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: methodcaller_missing_method_attributeerror (errors)."""
import operator

_raised = False
try:
    operator.methodcaller("no_such_method")(object())
except AttributeError:
    _raised = True
assert _raised, "methodcaller_missing_method_attributeerror: expected AttributeError"
print("methodcaller_missing_method_attributeerror OK")
