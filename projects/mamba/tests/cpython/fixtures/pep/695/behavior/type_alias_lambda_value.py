# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_alias_lambda_value"
# subject = "typing.TypeAliasType"
# kind = "semantic"
# xfail = "`type Lazy[T] = ...` does not bind the alias name on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeAliasType: an alias value may be a lambda closing over its own type param: type Lazy[T] = lambda: T has __value__() is that param"""


# An alias value may be a lambda closing over its own type param.
type Lazy[T] = lambda: T
lazy_t, = Lazy.__type_params__
assert Lazy.__value__() is lazy_t

print("type_alias_lambda_value OK")
