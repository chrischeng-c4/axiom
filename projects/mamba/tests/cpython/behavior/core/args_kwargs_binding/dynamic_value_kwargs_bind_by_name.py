# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "args_kwargs_binding"
# dimension = "behavior"
# case = "dynamic_value_kwargs_bind_by_name"
# subject = "dynamic function and bound-method kwargs binding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "mamba issue #1432"
# status = "filled"
# ///
"""Dynamic callable values bind kwargs by declared parameter name."""


def three(a, b, c):
    return [a, b, c]


alias = three
assert alias(**{"a": 1, "b": 2, "c": 3}) == [1, 2, 3]


class D:
    def g(self, __arg):
        return __arg

    def collect(self, **kwargs):
        return kwargs["x"]


closure = D().g
assert closure(_D__arg=5) == 5

try:
    closure(_WRONG=2)
except TypeError:
    pass
else:
    raise AssertionError("unknown keyword did not raise TypeError")

collector = D().collect
assert collector(x=8) == 8

print("dynamic_value_kwargs_bind_by_name OK")
