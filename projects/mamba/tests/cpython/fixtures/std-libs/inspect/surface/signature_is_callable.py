# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "signature_is_callable"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_is_callable (surface)."""
import inspect

assert callable(inspect.signature)
print("signature_is_callable OK")
