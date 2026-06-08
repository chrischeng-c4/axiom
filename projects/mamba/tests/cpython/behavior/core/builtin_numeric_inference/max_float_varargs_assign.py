# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "builtin_numeric_inference"
# dimension = "behavior"
# case = "max_float_varargs_assign"
# subject = "max() over float varargs, assigned then used"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""max(a, b, c) of float varargs assigned to a variable must yield the correct float value."""

m = max(2.25, 9.75, 4.5)
assert m == 9.75, m
assert isinstance(m, float), type(m)
diff = m - 0.75
assert diff == 9.0, diff
print("max_float_varargs_assign OK")
