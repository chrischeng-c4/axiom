# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_var_positional_and_var_keyword_kinds"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature() classifies *args as VAR_POSITIONAL and **kwargs as VAR_KEYWORD"""
import inspect

def _variadic(*args, **kwargs):
    pass

_vp = inspect.signature(_variadic).parameters
assert "args" in _vp, "args in variadic"
assert "kwargs" in _vp, "kwargs in variadic"
assert _vp["args"].kind == inspect.Parameter.VAR_POSITIONAL, "VAR_POSITIONAL"
assert _vp["kwargs"].kind == inspect.Parameter.VAR_KEYWORD, "VAR_KEYWORD"

print("signature_var_positional_and_var_keyword_kinds OK")
