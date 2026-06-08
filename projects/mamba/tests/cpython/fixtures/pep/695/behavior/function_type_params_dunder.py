# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "function_type_params_dunder"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "fn.__type_params__ returns None on mamba (type params not recorded; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a generic function exposes its params via __type_params__; nested def outer[A,B] / inner[C,D] each record their own (a,b) and (c,d)"""


# Generic functions expose their type params via __type_params__.
def outer[A, B]():
    def inner[C, D]():
        return (A, B, C, D)
    return inner


inner = outer()
a, b, c, d = inner()
assert outer.__type_params__ == (a, b)
assert inner.__type_params__ == (c, d)

print("function_type_params_dunder OK")
