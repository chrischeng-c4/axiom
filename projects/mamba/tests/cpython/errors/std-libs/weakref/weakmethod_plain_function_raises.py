# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakmethod_plain_function_raises"
# subject = "weakref.WeakMethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakMethod: weakmethod_plain_function_raises (errors)."""
import weakref

def _meth():
    return 1

_raised = False
try:
    weakref.WeakMethod(_meth)
except TypeError:
    _raised = True
assert _raised, "weakmethod_plain_function_raises: expected TypeError"
print("weakmethod_plain_function_raises OK")
