# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_not_name_mangled"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "Mangled.__type_params__ returns None on mamba so the unmangled names can't be read (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: type-param names are NOT mangled: a class Mangled[__T] keeps the literal '__T' name on its param, its method's __U, and its alias's __V"""


# Type param names are NOT mangled: a `__T` param keeps its literal name.
class Mangled[__T]:
    def meth[__U](self, arg: __T, arg2: __U):
        return (__T, __U)
    type Alias[__V] = (__T, __V)


assert Mangled.__type_params__[0].__name__ == "__T"
assert Mangled.meth.__type_params__[0].__name__ == "__U"
assert Mangled.Alias.__type_params__[0].__name__ == "__V"

print("type_params_not_name_mangled OK")
