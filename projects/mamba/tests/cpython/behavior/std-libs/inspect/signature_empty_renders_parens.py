# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_empty_renders_parens"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: an empty Signature() and the signature of a no-arg lambda both render as '()'"""
import inspect

S = inspect.Signature
assert str(S()) == "()", f"empty sig = {str(S())!r}"
assert str(inspect.signature(lambda: None)) == "()", "empty lambda sig"

print("signature_empty_renders_parens OK")
