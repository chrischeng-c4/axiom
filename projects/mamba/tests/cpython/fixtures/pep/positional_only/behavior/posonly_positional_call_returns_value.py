# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "posonly_positional_call_returns_value"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a positional-only function called positionally returns the computed value: def f(a, b, /): return a + b; f(1, 2) == 3"""

# Rule: positional-only params are bound by position and the call returns the
# computed value.
def _fn(a: int, b: int, /) -> int:
    return a + b

assert _fn(1, 2) == 3, _fn(1, 2)

print("posonly_positional_call_returns_value OK")
