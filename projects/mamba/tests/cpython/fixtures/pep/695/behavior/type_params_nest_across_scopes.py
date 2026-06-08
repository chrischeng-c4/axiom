# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_nest_across_scopes"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "A.__type_params__ returns None and nested type params are not captured on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: type params nest: an inner generic (class A[X] -> def b[Y] -> class C[Z] -> def d[W]) can see every enclosing param via a returned closure"""


# Type params nest: an inner generic can see every enclosing param.
class A[X]:
    def b[Y](self):
        class C[Z]:
            def d[W](self):
                return lambda: (X, Y, Z, W)
        return C


x_var, = A.__type_params__
y_var, = A.b.__type_params__
c_cls = A().b()
z_var, = c_cls.__type_params__
w_var, = c_cls.d.__type_params__
assert c_cls().d()() == (x_var, y_var, z_var, w_var)

print("type_params_nest_across_scopes OK")
