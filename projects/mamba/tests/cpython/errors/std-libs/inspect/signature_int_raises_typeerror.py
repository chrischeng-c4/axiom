# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "signature_int_raises_typeerror"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_int_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.signature(42)
except TypeError:
    _raised = True
assert _raised, "signature_int_raises_typeerror: expected TypeError"
print("signature_int_raises_typeerror OK")
