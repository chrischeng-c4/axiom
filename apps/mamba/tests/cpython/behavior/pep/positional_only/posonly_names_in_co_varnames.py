# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "posonly_names_in_co_varnames"
# subject = "/"
# kind = "semantic"
# xfail = "mamba function introspection returns None for fn.__code__, so co_varnames is unreadable (project_mamba_function_machinery_silent_divergences #8)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: positional-only parameter names still appear in __code__.co_varnames"""

# Rule: positional-only names are real locals, so they remain visible in the
# code object's co_varnames even though they cannot be named at the call site.
def _fn(a: int, b: int, /) -> int:
    return a + b

assert "a" in _fn.__code__.co_varnames, _fn.__code__.co_varnames
assert "b" in _fn.__code__.co_varnames, _fn.__code__.co_varnames

print("posonly_names_in_co_varnames OK")
