# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_param_name_collision_distinct"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "shadow.__type_params__ / Outer.meth.__type_params__ return None on mamba so the TypeVar identities can't be compared (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a type-param name may collide with a parameter name (def shadow[Arg](Arg)) and same-named params on an outer class and inner method are distinct objects"""


# A type param name may collide with a parameter name without conflict.
def shadow[Arg](Arg):
    return Arg


assert shadow(7) == 7
assert shadow.__type_params__[0].__name__ == "Arg"


# Same-named params on an outer class and inner method are distinct objects.
class Outer[T]:
    def meth[T](self):
        ...


outer_t, = Outer.__type_params__
inner_t, = Outer.meth.__type_params__
assert outer_t is not inner_t
assert outer_t.__name__ == inner_t.__name__ == "T"

print("type_param_name_collision_distinct OK")
