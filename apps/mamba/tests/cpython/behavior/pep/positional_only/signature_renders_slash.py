# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "signature_renders_slash"
# subject = "/"
# kind = "semantic"
# xfail = "mamba function introspection returns None for fn.__code__, so inspect.signature does not reflect the positional-only `/` (project_mamba_function_machinery_silent_divergences #8)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: inspect.signature renders the positional-only `/` separator: str(inspect.signature(f)) contains '/'"""
import inspect

# Rule: inspect.signature surfaces the positional-only marker as a `/` in the
# rendered signature string.
def _fn(a: int, b: int, /) -> int:
    return a + b

_sig = str(inspect.signature(_fn))
assert "/" in _sig, _sig

print("signature_renders_slash OK")
