# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "behavior"
# case = "kwargs_shadows_posonly_name"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a keyword colliding with a positional-only param name lands in **kwargs and does not shadow it: def f(a, /, **kw); f(1, a=999) binds a==1 and kw=={'a': 999}"""

# Rule: a keyword whose name matches a positional-only param does NOT conflict;
# it is captured by **kwargs instead of binding the positional-only parameter.
def _shadowed(a: int, /, **kwargs) -> dict:
    return {"a": a, "kwargs": kwargs}

_r = _shadowed(1, a=999)
assert _r["a"] == 1, _r["a"]
assert _r["kwargs"] == {"a": 999}, _r["kwargs"]

print("kwargs_shadows_posonly_name OK")
