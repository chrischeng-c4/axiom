# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "signature_unhashable_default_hash_raises_typeerror"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_unhashable_default_hash_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    hash(inspect.signature(lambda a={}: None))
except TypeError:
    _raised = True
assert _raised, "signature_unhashable_default_hash_raises_typeerror: expected TypeError"
print("signature_unhashable_default_hash_raises_typeerror OK")
