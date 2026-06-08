# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_str_positional_only_marker"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: str(signature) renders the positional-only '/' marker correctly"""
import inspect

def f2(a_po, /, *, b, **kwargs):
    pass

assert str(inspect.signature(f2)) == "(a_po, /, *, b, **kwargs)", "posonly str"

print("signature_str_positional_only_marker OK")
