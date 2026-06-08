# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakvaluedictionary_int_value_raises"
# subject = "weakref.WeakValueDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: weakvaluedictionary_int_value_raises (errors)."""
import weakref

_wvd = weakref.WeakValueDictionary()

_raised = False
try:
    _wvd.__setitem__('k', 42)
except TypeError:
    _raised = True
assert _raised, "weakvaluedictionary_int_value_raises: expected TypeError"
print("weakvaluedictionary_int_value_raises OK")
