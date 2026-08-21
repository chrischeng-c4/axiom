# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "typevar_bound_lazy_forward_ref"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: bounds/constraints are evaluated lazily: class LazyBound[T: Undefined, U: (Undefined,)] raises NameError on T.__bound__ until Undefined is defined, then resolves to 'defined'"""


# Bounds/constraints are evaluated lazily, so a forward (later-defined) name works.
class LazyBound[T: Undefined, U: (Undefined,)]:
    pass


params = LazyBound.__type_params__
# Before Undefined exists, touching the bound raises NameError lazily.
try:
    params[0].__bound__
    raise AssertionError("expected NameError for undefined bound")
except NameError:
    pass
assert params[1].__bound__ is None  # constrained -> no bound, no eval yet

Undefined = "defined"
# Now the lazy evaluation succeeds.
assert params[0].__bound__ == "defined"
assert params[1].__constraints__ == ("defined",)

print("typevar_bound_lazy_forward_ref OK")
