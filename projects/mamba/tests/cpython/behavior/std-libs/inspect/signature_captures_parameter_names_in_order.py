# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_captures_parameter_names_in_order"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature() lists positional/keyword parameter names in declaration order"""
import inspect

def _func(a, b, c=3):
    return a + b + c

_sig = inspect.signature(_func)
_names = list(_sig.parameters.keys())
assert _names == ["a", "b", "c"], f"param names = {_names!r}"

print("signature_captures_parameter_names_in_order OK")
