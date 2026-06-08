# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "custom_deepcopy_hook_propagates"
# subject = "copy.deepcopy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: custom_deepcopy_hook_propagates (errors)."""
import copy

_raised = False
try:
    copy.deepcopy(type('BadDeepCopy', (), {'__deepcopy__': lambda self, memo: (_ for _ in ()).throw(copy.Error('refused'))})())
except copy.Error:
    _raised = True
assert _raised, "custom_deepcopy_hook_propagates: expected copy.Error"
print("custom_deepcopy_hook_propagates OK")
