# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "debug_eq_emits_expr_and_value"
# subject = "fstring.debug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.debug: {expr=} emits the source text, '=', then the value (repr by default): with x=10, f'{x=}' is 'x=10' and f'{name=}' is "name='world'" for name='world'"""
# the =-debug form echoes the expression source plus its value

x = 10
assert f"{x=}" == "x=10"
name = "world"
assert f"{name=}" == "name='world'"

print("debug_eq_emits_expr_and_value OK")
