# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_str_renders_defaults_kwonly_varargs_return"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: str(signature) renders defaults, the keyword-only marker, *args/**kwargs, and the return annotation"""
import inspect

def f1(a: int = 1, *, b, c=None, **kwargs) -> 42:
    pass

assert str(inspect.signature(f1)) == "(a: int = 1, *, b, c=None, **kwargs) -> 42", (
    f"f1 str = {str(inspect.signature(f1))!r}"
)

print("signature_str_renders_defaults_kwonly_varargs_return OK")
