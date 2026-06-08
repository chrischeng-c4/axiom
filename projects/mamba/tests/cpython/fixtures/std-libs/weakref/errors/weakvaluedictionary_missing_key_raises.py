# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "weakvaluedictionary_missing_key_raises"
# subject = "weakref.WeakValueDictionary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: weakvaluedictionary_missing_key_raises (errors)."""
import weakref

_wvd = weakref.WeakValueDictionary()

_raised = False
try:
    _wvd['missing']
except KeyError:
    _raised = True
assert _raised, "weakvaluedictionary_missing_key_raises: expected KeyError"
print("weakvaluedictionary_missing_key_raises OK")
