# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_default_and_empty_sentinel"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter.default reflects the declared default; a parameter without one reports inspect.Parameter.empty"""
import inspect

def _func(a, b, c=3):
    return a + b + c

_params = inspect.signature(_func).parameters
assert _params["c"].default == 3, f"default c = {_params['c'].default!r}"
assert _params["a"].default is inspect.Parameter.empty, "a has no default"

print("parameter_default_and_empty_sentinel OK")
