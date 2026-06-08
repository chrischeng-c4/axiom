# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "getstate_exception_propagates"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: getstate_exception_propagates (errors)."""
import copy

_raised = False
try:
    copy.copy(type('EvilState', (), {'__getstate__': lambda self: (_ for _ in ()).throw(ValueError('no state'))})())
except ValueError:
    _raised = True
assert _raised, "getstate_exception_propagates: expected ValueError"
print("getstate_exception_propagates OK")
