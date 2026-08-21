# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "errors"
# case = "custom_copy_hook_propagates"
# subject = "copy.copy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: custom_copy_hook_propagates (errors)."""
import copy

_raised = False
try:
    copy.copy(type('BadCopy', (), {'__copy__': lambda self: (_ for _ in ()).throw(copy.Error('refused'))})())
except copy.Error:
    _raised = True
assert _raised, "custom_copy_hook_propagates: expected copy.Error"
print("custom_copy_hook_propagates OK")
