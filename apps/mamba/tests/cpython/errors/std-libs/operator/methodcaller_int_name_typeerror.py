# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_int_name_typeerror"
# subject = "operator.methodcaller"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: methodcaller_int_name_typeerror (errors)."""
import operator

_raised = False
try:
    operator.methodcaller(12)
except TypeError:
    _raised = True
assert _raised, "methodcaller_int_name_typeerror: expected TypeError"
print("methodcaller_int_name_typeerror OK")
