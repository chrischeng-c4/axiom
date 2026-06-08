# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "signature_type_raises_valueerror"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_type_raises_valueerror (errors)."""
import inspect

_raised = False
try:
    inspect.signature(type)
except ValueError:
    _raised = True
assert _raised, "signature_type_raises_valueerror: expected ValueError"
print("signature_type_raises_valueerror OK")
