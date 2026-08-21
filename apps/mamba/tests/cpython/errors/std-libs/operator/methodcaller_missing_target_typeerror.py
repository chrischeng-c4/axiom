# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_missing_target_typeerror"
# subject = "operator.methodcaller"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: calling a methodcaller with no target object raises TypeError (exactly one positional target is required)"""
import operator

caller = operator.methodcaller("upper")
_raised = False
try:
    caller()
except TypeError:
    _raised = True
assert _raised, "expected TypeError for missing target"
print("methodcaller_missing_target_typeerror OK")
