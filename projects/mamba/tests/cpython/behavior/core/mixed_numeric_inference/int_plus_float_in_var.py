# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "mixed_numeric_inference"
# dimension = "behavior"
# case = "int_plus_float_in_var"
# subject = "int + float through named variables"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int + float bound through variables promotes to the correct float (3 + 0.5 == 3.5)."""
a = 3
b = 0.5
s = a + b
assert s == 3.5, s
assert isinstance(s, float), type(s)
print("int_plus_float_in_var OK")
