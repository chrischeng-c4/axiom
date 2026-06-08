# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "arithmetic_matches_builtin_ops"
# subject = "operator.add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.add: binary arithmetic functions (add/sub/mul/truediv/floordiv/mod/pow) match the built-in operators exactly over representative integer inputs"""
import operator

assert operator.add(100, 200) == 300, "add"
assert operator.sub(100, 37) == 63, "sub"
assert operator.mul(12, 13) == 156, "mul"
assert operator.truediv(22, 7) == 22 / 7, "truediv"
assert operator.floordiv(22, 7) == 3, f"floordiv = {operator.floordiv(22, 7)!r}"
assert operator.mod(22, 7) == 1, "mod"
assert operator.pow(3, 4) == 81, "pow"

print("arithmetic_matches_builtin_ops OK")
